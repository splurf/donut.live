mod constants;
mod donut;
mod error;
mod handler;

use {
    constants::DELAY,
    donut::*,
    error::*,
    handler::*,
    std::{
        collections::HashMap,
        env::args,
        io::Write,
        net::{IpAddr, TcpListener, TcpStream, ToSocketAddrs},
        thread::sleep,
    },
};

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
    server.set_nonblocking(true)?;

    //  generate the donuts
    let frames = donuts();

    let mut streams: HashMap<IpAddr, TcpStream> = Default::default();
    let mut disconnected: Vec<IpAddr> = Default::default();

    loop {
        for frame in frames.iter() {
            //  handle any potential stream waiting to be accepted by the server
            if let Ok((stream, addr)) = server.accept() {
                if let Err(e) = handle_stream(stream, addr.ip(), &mut streams) {
                    eprintln!("{}", e)
                }
            }

            //  send each stream the current frame
            for (ip, mut stream) in streams.iter().clone() {
                //  fails if the stream disconnected from the server
                if let Err(_) = stream.write_all(frame.as_slice()) {
                    disconnected.push(*ip)
                }
            }

            //  remove every disconnected stream
            while let Some(ip) = disconnected.pop() {
                streams.remove(&ip);
            }
            sleep(DELAY)
        }
    }
}
