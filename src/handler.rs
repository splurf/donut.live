use {
    super::{
        constants::{CLEAR, GREED},
        error::*,
    },
    httparse::{Request, EMPTY_HEADER},
    std::{
        collections::HashMap,
        io::{Read, Write},
        net::{IpAddr, Shutdown, TcpStream},
    },
};

/** Verify the potential stream by checking if the User-Agent's product is `curl` and a few other practicalities */
fn verify_stream(mut stream: &TcpStream) -> Result<()> {
    let mut buf = [0; 128];
    _ = stream.read(&mut buf)?;

    let mut headers = [EMPTY_HEADER; 8];
    let mut req = Request::new(&mut headers);
    _ = req.parse(&buf)?;

    if let (Some(method), Some(path), Some(version)) = (req.method, req.path, req.version) {
        let user_agent = headers
            .iter()
            .find_map(|h| (h.name == "User-Agent").then_some(h.value))
            .ok_or("Invalid `User-Agent` header value")?;
        let accept = headers
            .iter()
            .find_map(|h| (h.name == "Accept").then_some(h.value))
            .ok_or("Invalid `Accept` header value")?;

        if method == "GET"
            && path == "/"
            && version == 1
            && user_agent.starts_with(b"curl")
            && accept == b"*/*"
        {
            Ok(())
        } else {
            Err("Invalid client properties".into())
        }
    } else {
        Err("Invalid client format".into())
    }
}

pub fn handle_stream(
    mut stream: TcpStream,
    ip: IpAddr,
    streams: &mut HashMap<IpAddr, TcpStream>,
) -> Result<()> {
    verify_stream(&stream)?;

    if streams.contains_key(&ip) {
        /* Send the refused stream a goodbye message then shutdown the connection */
        stream.write_all(&GREED)?;
        stream.shutdown(Shutdown::Both).map_err(Into::into)
    } else {
        /* Clear the screen of the stream and add them to the list of streams */
        stream.write_all(&CLEAR)?;
        streams.insert(ip, stream);
        Ok(())
    }
}
