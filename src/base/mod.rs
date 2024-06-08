mod cfg;
mod consts;
mod err;
mod sync;
mod utils;

use consts::*;
use sync::*;
use utils::*;

pub use cfg::*;
pub use err::*;

use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    thread::{sleep, JoinHandle},
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
    path: &str,
) -> JoinHandle<Result<()>> {
    let path = path.to_owned();

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

/// Distribute each frame to every stream.
fn dist_handler(
    streams: &CondLock<HashMap<SocketAddr, TcpStream>>,
    disconnected: &CondLock<Vec<SocketAddr>>,
    frames: &[Vec<u8>],
) -> Result<()> {
    // wait if and only if there are no connections
    streams.wait()?;

    // distribute the frames to each client
    for frame in frames {
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
}

/// Inititate program.
pub fn init() -> Result<()> {
    // parse program arguments
    let cfg = Config::new();

    // init listener
    let server = TcpListener::bind(cfg.addr())?;

    // connected clients
    let streams = CondLock::default();

    // disconnected clients
    let disconnected = CondLock::default();

    // init handlers
    error_handler(streams.clone(), disconnected.clone());
    incoming_handler(server, streams.clone(), cfg.path());

    // obtain frames
    let frames = donuts();

    println!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    // Distribute frames to each client as long as there is at least one connection.
    // Otherwise, the thread remains paused.
    loop_func(move || dist_handler(&streams, &disconnected, &frames))
}
