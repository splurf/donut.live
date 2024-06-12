mod ascii;
mod cfg;
mod donut;
mod err;
mod sync;
mod utils;

pub use ascii::*;
pub use cfg::*;
pub use err::*;
pub use sync::*;
pub use utils::*;

use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    thread::{sleep, JoinHandle},
};

/// The initial HTTP response headers appended with the `ESC[2J` erase function
const INIT: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n\x1b[2J";

/// Automatically remove any disconnected clients.
pub fn error_handler(
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
pub fn incoming_handler(
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
pub fn _dist_handler(
    streams: &CondLock<HashMap<SocketAddr, TcpStream>>,
    disconnected: &CondLock<Vec<SocketAddr>>,
    frame: &AsciiFrame,
) -> Result<()> {
    // discontinue distributing frames and pause
    // this thread if there are no connections
    if !*streams.lock()? {
        return Err(Error::Empty);
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
            if stream.write_all(frame.as_ref()).is_err() {
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
    Ok(())
}

/// Distribute each frame to every stream.
pub fn dist_handler(
    streams: &CondLock<HashMap<SocketAddr, TcpStream>>,
    disconnected: &CondLock<Vec<SocketAddr>>,
    frames: &[AsciiFrame],
    frame_index: &mut usize,
) -> Result<()> {
    // wait until there's at least one connection
    streams.wait()?;

    // distribute the frames to each client
    for (i, frame) in frames.iter().enumerate().skip(*frame_index) {
        println!("{}", i);
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
