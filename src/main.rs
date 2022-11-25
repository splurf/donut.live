mod donut;

use {
    donut::*,
    std::{
        collections::HashSet,
        env::args,
        io::{Read, Result, Write},
        net::{IpAddr, TcpListener, TcpStream, ToSocketAddrs},
        sync::{Arc, Mutex},
        thread::{sleep, spawn},
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

    let mut buf = [0; 96];
    let bytes = stream.read(&mut buf).ok()?;
    let curl = String::from_utf8_lossy(&buf[..bytes]);
    let parts = curl
        .split("\r\n")
        .filter(|p| !p.is_empty())
        .collect::<Vec<&str>>();

    (parts.len() == 4
        && {
            let (request, http) = parts[0].split_once(" / ")?;
            request == "GET" && {
                let (http, version) = http.split_once("/")?;
                http == "HTTP" && version == "1.1"
            }
        }
        && parts[2].split_once(" ")?.1.split_once("/")?.0 == "curl"
        && parts[3].split_once(" ")?.1 == "*/*")
        .then(|| (addr.ip(), stream))
}

/**
 * Continuously send each frame to the stream
 */
fn handle_stream(mut stream: TcpStream, frames: Vec<Vec<u8>>) -> Result<()> {
    stream.write_all(HEADER)?;

    loop {
        for frame in frames.iter() {
            stream.write_all(frame)?;
            sleep(DELAY);
        }
    }
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
    let frames = donuts();

    //  used to rate limit each stream to one session at a time
    let streams = Arc::new(Mutex::new(HashSet::<IpAddr>::default()));

    //  iterate through all incoming streams while verfying in the progress
    for (addr, stream) in server.incoming().filter_map(verify_stream) {
        //  insert the stream's address into `streams` then continue if it didn't previously exist
        if streams.lock().unwrap().insert(addr) {
            let streams_thread = streams.clone();

            //  the designated cloned frames for the verifed stream
            let frames_thread = frames.clone();

            //  have another thread handle the verified stream
            spawn(move || {
                let result = handle_stream(stream, frames_thread);
                //  remove stream`s address from `streams`
                streams_thread.lock().unwrap().remove(&addr);
                result
            });
        }
    }
    Ok(())
}
