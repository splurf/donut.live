mod cfg;
mod consts;
mod err;
mod sync;
mod util;

use {
    cfg::*,
    clap::Parser,
    consts::{DELAY, INIT},
    err::*,
    std::{
        collections::HashMap,
        io::Write,
        net::{SocketAddr, TcpListener, TcpStream},
        thread::{sleep, JoinHandle},
    },
    sync::*,
    util::*,
};

/// Automatically remove any disconnected clients.
fn error_handler(
    streams: CondLock<HashMap<SocketAddr, TcpStream>>,
    disconnected: CondLock<Vec<SocketAddr>>,
) -> JoinHandle<Result<()>> {
    init_handler(move || {
        // wait for a connection to be lost
        disconnected.wait()?;

        {
            // acquire and hold onto the write guards
            let mut gw_disconnected = disconnected.write()?;
            let mut gw_streams = streams.write()?;

            // remove every disconnected stream
            while let Some(ip) = gw_disconnected.pop() {
                gw_streams.remove(&ip);
            }
        }
        // update the predicate of `streams` so the main thread
        // knows when to pause due to there being no connections
        let is_empty = streams.read()?.is_empty();
        *streams.lock()? = !is_empty;

        // reset the boolean predicate
        *disconnected.lock()? = false;
        Ok(())
    })
}

/// Validate and instantiate streams into the system.
fn incoming_handler(
    server: TcpListener,
    streams: CondLock<HashMap<SocketAddr, TcpStream>>,
    path: String,
) -> JoinHandle<Result<()>> {
    init_handler(move || {
        // handle any potential stream waiting to be accepted by the server
        let (mut stream, addr) = server.accept()?;

        // determine the authenticity of the stream
        verify_stream(&stream, &path)?;

        // setup the client's terminal
        stream.write_all(INIT)?;

        // add the stream to the map
        streams.write()?.insert(addr, stream);

        // notify `streams` of a new connection
        *streams.lock()? = true;
        streams.notify();
        Ok(())
    })
}

fn main() -> Result<()> {
    // handle any program arguments
    let cfg = Config::parse();

    // instantiate the listener
    let server = TcpListener::bind(cfg.addr())?;

    // currently connected clients
    let streams = CondLock::<HashMap<SocketAddr, TcpStream>>::default();

    // clients that have disconnected
    let disconnected = CondLock::<Vec<SocketAddr>>::default();

    // init handlers
    _ = error_handler(streams.clone(), disconnected.clone());
    _ = incoming_handler(server, streams.clone(), cfg.path().to_string());

    // generate the original frames
    let mut frames = donuts(); // 559234 bytes

    // trim redundant whitespace
    trim_frames(&mut frames); // 395012 bytes (~29% smaller)

    println!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    // as long as there is at least one connection,
    // distribute the frames to each client, otherwise,
    // the thread remains paused.
    init_handler(move || {
        // wait if and only if there are no connections
        streams.wait()?;

        // distribute the frames to each client
        for frame in frames.iter() {
            // discontinue distributing frames and pause
            // this thread if there are no connections
            if !*streams.lock()? {
                break;
            }

            // the number of disconnections
            let res = {
                // acquire the writing guard of `disconnected`.
                // This doesn't cause a deadlock because `disconnected` is
                // only ever externally accessed after the end of this scope,
                // which is covered as this guard gets automatically dropped
                let mut g = disconnected.write()?;

                // send each stream the current frame
                for (ip, mut stream) in streams.read()?.iter() {
                    // remove the client if they have disconnected
                    if stream.write_all(frame).is_err() {
                        g.push(*ip);
                    }
                }
                // determinant for whether there have been any disconnections
                g.len() > 0
            };

            // notify `disconnected` due to a disconnection
            if res {
                *disconnected.lock()? = true;
                disconnected.notify();
            }

            // the pause between each frame
            sleep(DELAY)
        }
        Ok(())
    })
    .join()?
}
