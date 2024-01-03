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

/** Spawn a new thread that repeatedly calls the provided function until the break predicate is modified */
fn init_handler(
    mut f: impl FnMut(&mut bool) -> Result<()> + Send + 'static,
) -> JoinHandle<Result<()>> {
    spawn(move || -> Result<()> {
        let mut stop = false; // the "break" determinant

        loop {
            // Call the function
            if let Err(e) = f(&mut stop) {
                eprintln!("Error: {}", e)
            }
            // End loop if the determinant is true
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
        // Handle any potential stream waiting to be accepted by the server
        let (stream, addr) = server.accept()?;
        // Validate then setup the stream
        handle_stream(stream, addr.port(), streams.write()?, &path)?;
        // Notify `streams` of a new connection
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
        // Wait for a connection to be lost
        disconnected.wait()?;

        {
            // Acquire and hold onto the write guards
            let mut gw_disconnected = disconnected.write()?;
            let mut gw_streams = streams.write()?;

            // Remove every disconnected stream
            while let Some(ip) = gw_disconnected.pop() {
                gw_streams.remove(&ip);
            }
        }
        /*
         * Update the boolean predicate of `streams` so the main thread
         * knows when to pause due to there being no connections
         */
        let is_empty = streams.read()?.is_empty();
        *streams.lock()? = !is_empty;

        // Reset the boolean predicate
        *disconnected.lock()? = false;
        Ok(())
    })
}

fn main() -> Result<()> {
    // Handle any arguments
    let cfg = Config::parse();

    // Instantiate the listener
    let server = TcpListener::bind(cfg.addr())?;

    // Currently connected clients
    let streams = CondLock::<HashMap<u16, TcpStream>>::default();

    // Clients that have disconnected
    let disconnected = CondLock::<Vec<u16>>::default();

    // Instantiate handlers (external threads)
    let th1 = incoming_handler(server, streams.clone(), cfg.path().to_string());
    let th2 = error_handler(streams.clone(), disconnected.clone());

    // Original frames
    let mut frames = donuts(); // 559234 bytes

    // Trim the majority of the unnecessary whitespace
    trim_frames(&mut frames); // 395012 bytes (~29% smaller)

    println!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    /*
     * This is essentially the main thread
     *
     * As long as there is at least one connection,
     * distribute the frames to each client, otherwise,
     * put the thread to sleep.
     */
    init_handler(move |stop| {
        /*
         * If either of these threads have finished,
         * end the main thread because an
         * unexpected error has occurred
         */
        if th1.is_finished() || th2.is_finished() {
            *stop = true;
            Ok(())
        } else {
            // Wait if and only if there are no connections
            streams.wait()?;

            // Distribute the frames to each client
            for frame in frames.iter() {
                /*
                 * Discontinue distributing frames and pause
                 * this thread if there are no connections
                 */
                if !*streams.lock()? {
                    break;
                }

                if {
                    /*
                     * Acquire the writing guard of `disconnected`
                     * This doesn't cause a deadlock because `disconnected` is
                     * only ever externally accessed after the end of this scope,
                     * which is covered as this guard gets automatically dropped
                     */
                    let mut g = disconnected.write()?;

                    // Send each stream the current frame
                    for (ip, mut stream) in streams.read()?.iter() {
                        // Completely remove the client if they have disconnected
                        if stream.write_all(frame).is_err() {
                            g.push(*ip);
                        }
                    }
                    // Determinant for whether there have been any disconnections
                    g.len() > 0
                } {
                    /*
                     * This only happens if there was at least one disconnection.
                     *
                     * Have `th2` handle the disconnections, while
                     * this thread sleeps for the normal duration
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
