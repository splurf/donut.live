use std::{env::args, net::ToSocketAddrs};

mod donut;

use {
    donut::*,
    std::{
        io::{Read, Result, Write},
        net::{TcpListener, TcpStream},
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
fn verify_stream(stream: Result<TcpStream>) -> Option<TcpStream> {
    let mut stream = stream.ok()?;

    let mut buf = [0; 100];
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
        .then(|| stream)
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
    //  retrieve specfied address or resort to `localhost`
    let addr = {
        if let Some(arg) = args().nth(1) {
            arg
        } else {
            "localhost:80".to_string()
        }
        .to_socket_addrs()?
        .find(|sa| sa.port() == 80)
        .expect("Invalid Port (must use 80)")
    };

    //  initiate the listener
    let server = TcpListener::bind(addr)?;

    //  generate the donuts
    let frames = donuts();

    //  iterate through all incoming streams while verfying in the progress
    for stream in server.incoming().filter_map(verify_stream) {
        println!("{:?}", stream);

        //  the designated cloned frames for the verifed stream
        let frames_thread = frames.clone();

        //  have another thread handle the verified stream
        spawn(|| {
            if let Err(e) = handle_stream(stream, frames_thread) {
                println!("{}", e)
            }
        });
    }
    Ok(())
}
