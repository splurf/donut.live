mod donut;

use {
    donut::*,
    std::{
        collections::HashMap,
        env::args,
        io::{Error, Read, Write},
        net::{IpAddr, Shutdown, TcpListener, TcpStream, ToSocketAddrs},
        sync::{Arc, Mutex},
        thread::{scope, sleep},
        time::Duration,
    },
};

/**
 * The header that each verfied curl request will receive
 */
const HEADER: &'static str = "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n";

/**
 * The ASCII call for clearing the terminal screen
 */
const CLEAR: &'static str = "\x1b[2J";

/**
 * What is said to a client before they are rejected
 * for trying to curl multiple donuts at once
 */
const GREED: &'static str = "Don't be greedy...";

/**
 * The delay between each frame
 */
const DELAY: Duration = Duration::from_millis(16);

/**
 * Simple helper function for concatenating
 * the body to the default response
 */
fn body(s: &'static str) -> Vec<u8> {
    format!("{}{}", HEADER, s).into_bytes()
}

/**
 * Verify the potential stream by checking if the User-Agent's product is `curl` and a few other practicalities
 */
fn verify_stream(stream: Result<TcpStream, Error>) -> Option<(IpAddr, TcpStream)> {
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
fn handle_stream(
    mut stream: impl Write,
    frames: &[Box<[u8; 1784]>],
    frame_index: &Mutex<usize>,
) -> Result<(), Error> {
    stream.write_all(&body(CLEAR))?;

    //  Aquire the lock and hold it until the connection is lost
    let mut frame_index_guard = frame_index.lock().unwrap();

    //  Place the frames into a cycle then advance the iterator to the last visited frame
    let mut frames_iter = frames.iter().enumerate().cycle();
    frames_iter.nth(*frame_index_guard);

    for (i, frame) in frames_iter {
        //  If the stream loses connection, set the frame index to the index of the current frame
        if let Err(e) = stream.write_all(frame.as_slice()) {
            *frame_index_guard = i;
            return Err(e);
        }
        sleep(DELAY);
    }
    unreachable!("iterator is infinite")
}

fn main() -> Result<(), String> {
    //  retrieve specfied address or default to `localhost`
    let addr = args()
        .nth(1)
        .unwrap_or("localhost:80".to_string())
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("Invalid provided address")?;

    //  initiate the listener
    let server = TcpListener::bind(addr).map_err(|e| e.to_string())?;

    //  generate the donuts
    let frames = &donuts();

    //  used to rate limit each stream to one session at a time
    let streams: Mutex<HashMap<IpAddr, Arc<Mutex<usize>>>> = Default::default();

    scope(|s| -> Result<(), String> {
        //  iterate through all incoming streams while verfying in the progress
        for (ip, mut stream) in server.incoming().filter_map(verify_stream) {
            //  retrieve the lock for the existing `frame_index` of the stream or insert a new one starting at 0 and use that instead
            let frame_index = {
                let mut guard = streams.lock().map_err(|e| e.to_string())?;
                let entry = guard.entry(ip);
                entry.or_default().clone()
            };

            //  if the lock of the `frame_index` of this stream is available then continue
            if frame_index.try_lock().is_ok() {
                //  have another thread handle the verified stream while ignoring the result
                let _ = s.spawn(move || handle_stream(stream, frames, &frame_index));
            } else {
                //  send the refused stream a goodbye message
                stream.write_all(&body(GREED)).map_err(|e| e.to_string())?;
                stream.shutdown(Shutdown::Both).map_err(|e| e.to_string())?;
            }
        }
        unreachable!("iterator is infinite")
    })
}
