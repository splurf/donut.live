mod donut;

use {
    donut::*,
    std::{
        collections::HashSet,
        env::args,
        io::{Read, Result, Write},
        net::{IpAddr, TcpListener, TcpStream, ToSocketAddrs},
        sync::Mutex,
        thread::{scope, sleep},
        time::Duration,
    },
};

/**
 * The header that each verfied curl request will receive
 * `\x1b[2J` is appended for convenience
 */
const HEADER: &[u8; 64] =
    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n\x1b[2J";

/**
 * The delay between each frame
 */
const DELAY: Duration = Duration::from_millis(16);

/**
 * Verify the potential stream by checking if the User-Agent's product is `curl` and a few other practicalities
 */
fn verify_stream(stream: Result<TcpStream>) -> Option<(IpAddr, TcpStream)> {
    let mut stream = stream.ok()?;
    let addr = stream.peer_addr().ok()?;

    let mut buf = [0; 128];
    let bytes = stream.read(&mut buf).ok()?;
    let curl = String::from_utf8_lossy(&buf[..bytes]);

    //  retrieve the necessary headers
    if let [premise, .., user_agent, accept] = curl
        .split("\r\n")
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()[..]
    {
        //  verify those headers
        (premise == "GET / HTTP/1.1"
            && user_agent.starts_with("User-Agent: curl/")
            && accept == "Accept: */*")
            .then_some((addr.ip(), stream))
    } else {
        None
    }
}

/**
 * Continuously send each frame to the stream
 */
fn handle_stream(mut stream: impl Write, frames: &[Vec<u8>]) -> Result<()> {
    stream.write_all(HEADER)?;

    for frame in frames.iter().cycle() {
        stream.write_all(frame)?;
        sleep(DELAY);
    }
    unreachable!("iterator is infinite")
}

fn main() -> Result<()> {
    //  retrieve specfied address or default to `localhost`
    let addr = {
        args()
            .nth(1)
            .unwrap_or("localhost:80".to_string())
            .to_socket_addrs()?
            .next()
            .expect("Invalid provided address")
    };

    //  initiate the listener
    let server = TcpListener::bind(addr)?;

    //  generate the donuts
    let frames = &donuts();

    //  used to rate limit each stream to one session at a time
    let streams: &Mutex<HashSet<IpAddr>> = &Default::default();

    scope(|s| {
        //  iterate through all incoming streams while verfying in the progress
        for (ip, stream) in server.incoming().filter_map(verify_stream) {
            //  insert the stream's address into `streams` then continue if it didn't previously exist
            if streams.lock().unwrap().insert(ip) {
                //  have another thread handle the verified stream
                s.spawn(move || {
                    //  this is ignored
                    let _ = handle_stream(stream, frames);
                    //  remove stream`s address from `streams`
                    streams.lock().unwrap().remove(&ip);
                });
            }
        }
        unreachable!("iterator is infinite")
    })
}
