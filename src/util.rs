use {
    super::{
        consts::{CHARACTERS, INIT},
        err::*,
    },
    httparse::{Request, EMPTY_HEADER},
    std::{
        collections::HashMap,
        f32::consts::TAU,
        io::{Read, Write},
        net::{Shutdown, TcpStream},
        sync::RwLockWriteGuard,
    },
};

/** Generate a single frame of the donut based on the given variables */
fn generate_frame(
    a: &mut f32,
    b: &mut f32,
    i: &mut f32,
    j: &mut f32,
    z: &mut [f32; 1760],
    p: &mut [u8; 1760],
) -> Vec<u8> {
    while *j < TAU {
        while *i < TAU {
            let c = f32::sin(*i);
            let d = f32::cos(*j);
            let e = f32::sin(*a);
            let f = f32::sin(*j);
            let g = f32::cos(*a);
            let h = d + 2.0;
            let q = 1.0 / (c * h * e + f * g + 5.0);
            let l = f32::cos(*i);
            let m = f32::cos(*b);
            let n = f32::sin(*b);
            let t = c * h * g - f * e;

            let x = (40.0 + 30.0 * q * (l * h * m - t * n)) as i32;
            let y = (12.0 + 15.0 * q * (l * h * n + t * m)) as i32;
            let o = x + 80 * y;
            let n = (8.0 * ((f * e - c * d * g) * m - c * d * e - f * g - l * d * n)) as i32;

            if 22 > y && y > 0 && x > 0 && 80 > x && q > z[o as usize] {
                z[o as usize] = q;
                p[o as usize] = CHARACTERS[if n > 0 { n } else { 0 } as usize]
            }
            *i += 0.02
        }
        *i = 0.0;
        *j += 0.07
    }
    *a += 0.04;
    *b += 0.02;

    let frame = p
        .chunks_exact(80)
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>()
        .join(&10);

    *p = [32; 1760];
    *z = [0.0; 1760];
    *j = 0.0;

    frame
}

/** *donut.c* refactored into rust */
pub fn donuts() -> [Vec<u8>; 314] {
    let mut a = 0.0;
    let mut b = 0.0;

    let mut i = 0.0;
    let mut j = 0.0;

    let mut z = [0.0; 1760];
    let mut p = [32; 1760];

    // generate the original `donut` frames
    [0; 314].map(|_| generate_frame(&mut a, &mut b, &mut i, &mut j, &mut z, &mut p))
}

/**
 * Trim the *majority* of the unnecessary whitespace at the end of each line of every frame
 * TODO - trim all redundant whitespace, without altering how the frames look
*/
pub fn trim_frames(frames: &mut [Vec<u8>; 314]) {
    // return the lines of each frame
    fn split<'a>(f: &'a [u8]) -> impl Iterator<Item = Vec<u8>> + 'a {
        f.split(|c| *c == 10).map(<[u8]>::to_vec)
    }

    // this is pretty disgusting
    //
    // determine the maximum length of non-ascii of each line for every frame
    let maxes = {
        let mut out = [[0; 314]; 22];

        for (j, chunk) in frames
            .iter()
            .map(|f| {
                split(f)
                    .map(|u| {
                        u.iter()
                            .rposition(|c| CHARACTERS.contains(c))
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
            .chunks(22)
            .enumerate()
        {
            for (i, n) in chunk.iter().enumerate() {
                out[i][j] = *n
            }
        }
        out.into_iter()
            .map(|u| u.into_iter().max().unwrap_or_default())
            .collect::<Vec<_>>()
    };

    // drain the ascii of each line for every frame for their max length
    frames.iter_mut().for_each(|f| {
        *f = split(f.as_slice())
            .enumerate()
            .map(|(i, l)| l[0..maxes[i]].to_vec())
            .collect::<Vec<Vec<u8>>>()
            .join(&10);
        f.splice(0..0, "\x1b[H".bytes());
    });
}

/** Verify the potential client by checking if the User-Agent's product is `curl` and a few other practicalities */
fn verify_stream(mut stream: &TcpStream, uri_path: &str) -> Result<()> {
    // read from the incoming stream
    let mut buf = [0; 128];
    let bytes = stream.read(&mut buf)?;

    // parse the request
    let mut headers = [EMPTY_HEADER; 8];
    let mut req = Request::new(&mut headers);
    _ = req.parse(&buf[..bytes])?;

    if let (Some(method), Some(path), Some(version)) = (req.method, req.path, req.version) {
        if method != "GET" {
            Err(UriError::Method(method.to_string()).into())
        } else if path != uri_path {
            Err(UriError::Path(path.to_string()).into())
        } else if version != 1 {
            Err(UriError::Version(version).into())
        } else if let Some(h) = req
            .headers
            .into_iter()
            .take_while(|h| !h.name.is_empty())
            .filter_map(|h| match h.name {
                "User-Agent" => (!h.value.starts_with(b"curl")).then_some(h),
                "Accept" => (h.value != b"*/*").then_some(h),
                _ => None,
            })
            .next()
        {
            Err(UriError::from(h).into())
        } else {
            Ok(())
        }
    } else {
        Err(Invalid::Format.into())
    }
}

pub fn handle_stream(
    mut stream: TcpStream,
    port: u16,
    mut streams: RwLockWriteGuard<HashMap<u16, TcpStream>>,
    path: &str,
) -> Result<()> {
    // determine the authenticity of the stream
    verify_stream(&stream, path)?;

    if streams.contains_key(&port) {
        stream.shutdown(Shutdown::Both)?; // close the connection
        Err(Invalid::DuplicateStream.into()) // this is fairly irregular
    } else {
        stream.write_all(&INIT)?; // clear the client's terminal
        streams.insert(port, stream); // add the client to the list of current streams
        Ok(())
    }
}
