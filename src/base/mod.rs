mod ascii;
mod cfg;
mod client;
mod donut;
mod err;
mod frame;
mod progress;
mod sync;
mod util;

pub use ascii::*;
pub use cfg::*;
pub use client::*;
pub use err::*;
pub use frame::*;
pub use progress::*;
pub use sync::*;
pub use util::*;

#[cfg(feature = "logger")]
mod logger;

#[cfg(feature = "logger")]
pub use logger::*;

use std::{
    io::Write,
    net::{SocketAddr, TcpListener},
    thread::{sleep, JoinHandle},
};

/// The initial HTTP response headers appended with the `ESC[2J` erase function
const INIT: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n\x1b[2J";

/// Automatically remove any disconnected clients.
pub fn error_handler(
    streams: SignalLock<Clients>,
    disconnected: SignalLock<Vec<SocketAddr>>,
) -> JoinHandle<Result> {
    init_handler(move || {
        // wait for a connection to be lost
        disconnected.wait();

        {
            // acquire and hold onto the write guards
            let mut gw_disconnected = disconnected.write();
            let mut gw_streams = streams.write();

            // remove every disconnected stream
            gw_disconnected.drain(..).for_each(|addr| {
                gw_streams.remove(&addr);
            });
        }
        // update the predicate of `streams` so the main thread
        // knows when to pause due to there being no connections
        let is_empty = streams.read().is_empty();
        *streams.lock() = !is_empty;

        // reset the boolean predicate
        *disconnected.lock() = false;
        Ok(())
    })
}

/// Validate and instantiate streams into the system.
pub fn incoming_handler(
    server: TcpListener,
    streams: SignalLock<Clients>,
    path: String,
) -> JoinHandle<Result> {
    init_handler(move || {
        // handle any potential stream waiting to be accepted by the server
        let (mut stream, ..) = server.accept()?;

        // determine the authenticity of the stream
        let addr = verify_stream(&stream, &path)?;

        // setup the client's terminal
        stream.write_all(INIT)?;

        // add the stream to the map
        streams.write().insert(addr, Client::new(stream, addr));

        // notify `streams` of a new connection
        *streams.lock() = true;
        streams.notify();

        #[cfg(feature = "logger")]
        log_to_file(addr)?;

        Ok(())
    })
}

/// Distribute each frame to every stream.
pub fn _dist_handler(
    streams: &SignalLock<Clients>,
    disconnected: &SignalLock<Vec<SocketAddr>>,
    frame: &AsciiFrame,
) -> Result {
    // discontinue distributing frames and pause
    // this thread if there are no connections
    if !*streams.lock() {
        return Err(Error::Empty);
    }

    // the number of disconnections
    let res = {
        // acquire the writing guard of `disconnected`.
        // This doesn't cause a deadlock because `disconnected` is
        // only ever externally accessed after the end of this scope,
        // which is covered as this guard gets automatically dropped
        let mut g = disconnected.write();

        // send each stream the current frame
        for mut client in streams.read().values() {
            // remove the client if they have disconnected
            if client.write_all(frame.as_ref()).is_err() {
                g.push(client.addr())
            }
        }
        // determinant for whether there have been any disconnections
        g.len() > 0
    };

    // notify `disconnected` due to a disconnection
    if res {
        *disconnected.lock() = true;
        disconnected.notify();
    }
    Ok(())
}

/// Distribute each frame to every stream.
pub fn dist_handler(
    streams: &SignalLock<Clients>,
    disconnected: &SignalLock<Vec<SocketAddr>>,
    frames: &[AsciiFrame],
    frame_index: &mut usize,
) -> Result {
    // wait until there's at least one connection
    streams.wait();

    // distribute the frames to each client
    for (i, frame) in frames.iter().enumerate().skip(*frame_index) {
        if let Err(e) = _dist_handler(streams, disconnected, frame) {
            *frame_index = i;
            return Err(e);
        }
        // the delay of the current frame
        sleep(frame.delay())
    }
    // reset frame index
    *frame_index = 0;
    Ok(())
}
