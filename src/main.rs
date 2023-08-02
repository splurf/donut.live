mod cfg;
mod consts;
mod err;
mod util;

use {
    cfg::*,
    clap::Parser,
    consts::DELAY,
    err::*,
    std::{
        collections::HashMap,
        io::Write,
        net::{IpAddr, TcpListener, TcpStream},
        thread::sleep,
    },
    util::*,
};

fn main() -> Result<()> {
    let cfg = Config::parse();

    //  initiate the listener
    let server = TcpListener::bind(cfg.addr())?;
    server.set_nonblocking(true)?;

    //  generate the donuts
    let frames = donuts();

    let mut streams: HashMap<IpAddr, TcpStream> = Default::default();
    let mut disconnected: Vec<IpAddr> = Default::default();

    println!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    loop {
        for frame in frames.iter() {
            //  handle any potential stream waiting to be accepted by the server
            if let Ok((stream, addr)) = server.accept() {
                if let Err(e) = handle_stream(stream, addr.ip(), &mut streams, cfg.path()) {
                    eprintln!("{}", e)
                }
            }

            //  send each stream the current frame
            for (ip, mut stream) in streams.iter() {
                //  fails if the stream disconnected from the server
                if let Err(_) = stream.write_all(frame) {
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
