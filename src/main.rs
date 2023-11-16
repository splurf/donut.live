mod cfg;
mod consts;
mod err;
mod sync;
mod util;

use {
    cfg::*,
    clap::Parser,
    consts::DELAY,
    err::*,
    std::{
        collections::HashMap,
        io::Write,
        net::{TcpListener, TcpStream},
        thread::{sleep, spawn, JoinHandle},
    },
    sync::*,
    util::*,
};

/** Validate and instantiate streams into the system */
fn incoming_handler(
    server: TcpListener,
    streams: CondLock<HashMap<u16, TcpStream>>,
    path: String,
) -> JoinHandle<Result<()>> {
    spawn(move || -> Result<()> {
        loop {
            // handle any potential stream waiting to be accepted by the server
            if let Ok((stream, addr)) = server.accept() {
                if let Err(e) = handle_stream(stream, addr.port(), streams.write()?, &path) {
                    eprintln!("{}", e)
                } else {
                    *streams.lock()? = true;
                    streams.notify_one();
                }
            }
        }
    })
}

/** Automatically remove any disconnected clients */
fn error_handler(
    streams: CondLock<HashMap<u16, TcpStream>>,
    disconnected: CondLock<Vec<u16>>,
) -> JoinHandle<Result<()>> {
    spawn(move || -> Result<()> {
        loop {
            // wait for at least one connection to be lost
            disconnected.wait()?;

            // remove every disconnected stream
            while let Some(ip) = disconnected.write()?.pop() {
                streams.write()?.remove(&ip);
                let is_empty = streams.read()?.is_empty();
                *streams.lock()? = !is_empty;
            }
            // reset the condition variable
            *disconnected.lock()? = false;
        }
    })
}

/** Distribute frames to every connected client */
fn main_thread(
    cfg: Config,
    streams: CondLock<HashMap<u16, TcpStream>>,
    disconnected: CondLock<Vec<u16>>,
    th1: JoinHandle<Result<()>>,
    th2: JoinHandle<Result<()>>,
) -> Result<()> {
    // original frames (stored in memory)
    let mut frames = donuts(); // 559234 bytes

    // trim the majority of the unnecessary whitespace
    trim_frames(&mut frames); // 395012 bytes (~29% smaller)

    println!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    loop {
        // if either of these threads have finished,
        // end the main thread because an unexpected
        // poison error as ocurred
        if th1.is_finished() || th2.is_finished() {
            break Ok(());
        }
        // wait until there is at least one connection
        streams.wait()?;

        // distribute frames to each client
        for frame in frames.iter() {
            // stop distributing if there are no connections
            if !*streams.lock()? {
                break;
            }

            // send each stream the current frame
            for (ip, mut stream) in streams.read()?.iter() {
                // completely remove the client if they have disconnected
                if let Err(_) = stream.write_all(frame) {
                    disconnected.write()?.push(*ip);
                    *disconnected.lock()? = true;
                    disconnected.notify_one()
                }
            }
            sleep(DELAY)
        }
    }
}

fn main() -> Result<()> {
    // handle any arguments
    let cfg = Config::parse();

    // instantiate the listener
    let server = TcpListener::bind(cfg.addr())?;

    // the list of currently connected clients
    let streams = CondLock::<HashMap<u16, TcpStream>>::default();

    // the list of clients that have disconnected
    let disconnected = CondLock::<Vec<u16>>::default();

    let th1 = incoming_handler(server, streams.clone(), cfg.path().to_string());
    let th2 = error_handler(streams.clone(), disconnected.clone());

    main_thread(cfg, streams, disconnected, th1, th2)
}
