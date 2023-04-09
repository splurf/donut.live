mod constants;
mod donut;
mod error;
mod handler;

use {
    donut::*,
    error::*,
    handler::*,
    httparse::{Request, EMPTY_HEADER},
    std::{
        collections::HashMap,
        env::args,
        io::Read,
        net::{IpAddr, TcpListener, TcpStream, ToSocketAddrs},
        sync::{Arc, Mutex},
        thread::scope,
    },
};

/** Verify the potential stream by checking if the User-Agent's product is `curl` and a few other practicalities */
fn verify_stream(stream: Result<TcpStream, std::io::Error>) -> Option<(IpAddr, TcpStream)> {
    let mut stream = stream.ok()?;
    let addr = stream.peer_addr().ok()?;

    let mut buf = [0; 128];
    _ = stream.read(&mut buf).ok()?;

    let mut headers = [EMPTY_HEADER; 8];
    let mut req = Request::new(&mut headers);
    _ = req.parse(&buf).ok()?;

    if let (Some(method), Some(path), Some(version)) = (req.method, req.path, req.version) {
        let user_agent = headers
            .iter()
            .find_map(|h| (h.name == "User-Agent").then_some(h.value))?;
        let accept = headers
            .iter()
            .find_map(|h| (h.name == "Accept").then_some(h.value))?;

        (method == "GET"
            && path == "/"
            && version == 1
            && user_agent.starts_with(b"curl")
            && accept == b"*/*")
            .then_some((addr.ip(), stream))
    } else {
        None
    }
}

fn main() -> Result<()> {
    //  retrieve specfied address or default to `localhost`
    let addr = args()
        .nth(1)
        .unwrap_or("localhost:80".into())
        .to_socket_addrs()?
        .next()
        .ok_or("Invalid provided address")?;

    //  initiate the listener
    let server = TcpListener::bind(addr)?;

    //  generate the donuts
    let frames = &donuts();

    //  used to rate limit each stream to one session at a time
    let streams: Mutex<HashMap<IpAddr, Arc<Mutex<usize>>>> = Default::default();

    scope(|s| -> Result<()> {
        //  iterate through all incoming streams while verfying in the progress
        for (ip, stream) in server.incoming().filter_map(verify_stream) {
            //  retrieve the lock for the existing `frame_index` of the stream or insert a new one starting at 0 and use that instead
            let frame_index = {
                let mut guard = streams.lock()?;
                let entry = guard.entry(ip);
                entry.or_default().clone()
            };

            //  move each unhandled stream to another thread
            _ = s.spawn(move || {
                //  if the lock of the `frame_index` of this stream is available then continue
                if frame_index.try_lock().is_ok() {
                    //  have another thread handle the verified stream while ignoring the result
                    handle_stream(stream, frames, &frame_index)
                } else {
                    //  shutdown the stream while igoring the result
                    close_stream(stream)
                }
            })
        }
        unreachable!("iterator is infinite")
    })
}
