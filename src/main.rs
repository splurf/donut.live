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

/** Spawn a new thread which repeatedly calls the provided function until the break predicate is modified */
fn init_handler(
    mut f: impl FnMut(&mut bool) -> Result<()> + Send + 'static,
) -> JoinHandle<Result<()>> {
    spawn(move || -> Result<()> {
        let mut stop = false; // break predicate
        loop {
            // call the function
            if let Err(e) = f(&mut stop) {
                eprintln!("Error: {}", e)
            }
            // end loop if break predicate is true
            if stop {
                break Ok(());
            }
        }
    })
}

/** Validate and instantiate streams into the system */
fn incoming_handler(
    server: TcpListener,
    streams: CondLock<HashMap<u16, TcpStream>>,
    path: String,
) -> JoinHandle<Result<()>> {
    init_handler(move |_| {
        // handle any potential stream waiting to be accepted by the server
        let (stream, addr) = server.accept()?;
        // validate then setup the stream
        handle_stream(stream, addr.port(), streams.write()?, &path)?;
        // notify `streams` of a new connection
        *streams.lock()? = true;
        streams.notify();
        Ok(())
    })
}

/** Automatically remove any disconnected clients */
fn error_handler(
    streams: CondLock<HashMap<u16, TcpStream>>,
    disconnected: CondLock<Vec<u16>>,
) -> JoinHandle<Result<()>> {
    init_handler(move |_| {
        // wait for a connection to be lost
        disconnected.wait()?;

        {
            // aquire and hold onto the write guards
            let mut gw_disconnected = disconnected.write()?;
            let mut gw_streams = streams.write()?;

            // remove every disconnected stream
            while let Some(ip) = gw_disconnected.pop() {
                gw_streams.remove(&ip);
            }
        }
        /*
         * update the boolean predicate of `streams` so the main thread
         * knows when to pause due to there being no connections
         */
        let is_empty = streams.read()?.is_empty();
        *streams.lock()? = !is_empty;

        // reset the boolean predicate
        *disconnected.lock()? = false;
        Ok(())
    })
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

    // original frames (stored in memory)
    let mut frames = donuts(); // 559234 bytes

    // trim the majority of the unnecessary whitespace
    trim_frames(&mut frames); // 395012 bytes (~29% smaller)

    println!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    /*
     * This is essentially the main thread
     *
     * When there are connections, distribute frames to every client,
     * otherwise, the thread sleeps.
     */
    init_handler(move |stop| {
        /*
         * if either of these threads have finished,
         * end the main thread because an unexpected
         * error as ocurred
         */
        if th1.is_finished() || th2.is_finished() {
            *stop = true;
            Ok(())
        } else {
            // wait if and only if there are no connections
            streams.wait()?;

            // distribute frames to each client
            for frame in frames.iter() {
                /*
                 * discontinue distributing frames and pause
                 * this thread if there are no connections
                 */
                if !*streams.lock()? {
                    break;
                }

                if {
                    /*
                     * aquire the writing guard of `disconnected`
                     * this doesn't cause a deadlock because `disconnected` is
                     * only ever externally accessed after the end of this scope,
                     * which is covered as this guard gets automatically dropped
                     */
                    let mut g = disconnected.write()?;

                    // send each stream the current frame
                    for (ip, mut stream) in streams.read()?.iter() {
                        // completely remove the client if they have disconnected
                        if stream.write_all(frame).is_err() {
                            g.push(*ip);
                        }
                    }
                    // determinant for whether there have been any disconnections
                    g.len() > 0
                } {
                    /*
                     * this only happens if there was at least one disconnections
                     * have the other thread (`th2`) handle the disconnections
                     * awhile this thread sleeps for the normal duration
                     */
                    *disconnected.lock()? = true;
                    disconnected.notify();
                }
                sleep(DELAY)
            }
            Ok(())
        }
    })
    .join()?
}
