mod base;

use log::trace;
use std::net::TcpListener;

use base::*;

fn main() -> Result {
    // parse program arguments
    let cfg = Config::new()?;

    // create log file if it does not already exist
    #[cfg(feature = "logger")]
    init_log_file();

    // retrieve ascii frames
    let frames = get_frames(&cfg)?;

    // init listener
    trace!("Initializing TCP server");
    let server = TcpListener::bind(cfg.addr())?;

    // connected clients
    let streams = SignalLock::default();

    // disconnected clients
    let disconnected = SignalLock::default();

    // init handlers
    error_handler(streams.clone(), disconnected.clone());
    incoming_handler(server, streams.clone(), cfg.path().to_owned());

    trace!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    // global frame index
    let mut frame_index = 0;

    // Distribute frames to each client as long as there is at least one connection.
    // Otherwise, the thread remains paused.
    loop_func(move || {
        // update frame index after every iteration
        dist_handler(&streams, &disconnected, &frames, &mut frame_index)
    })
}
