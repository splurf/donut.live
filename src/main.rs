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
<<<<<<< HEAD
        net::{TcpListener, TcpStream},
        sync::{Arc, Condvar, Mutex, RwLock},
        thread::{sleep, spawn, JoinHandle},
=======
        net::{IpAddr, TcpListener, TcpStream},
        thread::sleep,
>>>>>>> e13826e13ffea178880ea30ab20dd6cad2efd513
    },
    util::*,
};

<<<<<<< HEAD
fn wait(pair: &Arc<(Mutex<bool>, Condvar)>) -> Result<()> {
    let (g, cvar) = &**pair;
    let mut started = g.lock()?;
    while !*started {
        started = cvar.wait(started)?;
    }
    Ok(())
}

fn incoming_handler(
    server: TcpListener,
    streams: Arc<(RwLock<HashMap<u16, TcpStream>>, Arc<(Mutex<bool>, Condvar)>)>,
    path: String,
) -> JoinHandle<Result<()>> {
    spawn(move || -> Result<()> {
        loop {
            //  handle any potential stream waiting to be accepted by the server
            if let Ok((stream, addr)) = server.accept() {
                if let Err(e) = handle_stream(stream, addr.port(), streams.0.write()?, &path) {
=======
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
>>>>>>> e13826e13ffea178880ea30ab20dd6cad2efd513
                    eprintln!("{}", e)
                } else {
                    *streams.1 .0.lock()? = true;
                    streams.1 .1.notify_one();
                }
            }
        }
    })
}

fn error_handler(
    streams: Arc<(RwLock<HashMap<u16, TcpStream>>, Arc<(Mutex<bool>, Condvar)>)>,
    disconnected: Arc<(RwLock<Vec<u16>>, Arc<(Mutex<bool>, Condvar)>)>,
) -> JoinHandle<Result<()>> {
    spawn(move || -> Result<()> {
        loop {
            // wait for at least one connection to be lost
            wait(&disconnected.1)?;

            //  remove every disconnected stream
            while let Some(ip) = disconnected.0.write()?.pop() {
                streams.0.write()?.remove(&ip);
                let is_empty = streams.0.read()?.is_empty();
                *streams.1 .0.lock()? = !is_empty;
            }
            // reset the condition variable
            *disconnected.1 .0.lock()? = false;
        }
    })
}

fn main_thread(
    cfg: Config,
    streams: Arc<(RwLock<HashMap<u16, TcpStream>>, Arc<(Mutex<bool>, Condvar)>)>,
    disconnected: Arc<(RwLock<Vec<u16>>, Arc<(Mutex<bool>, Condvar)>)>,
    th1: JoinHandle<Result<()>>,
    th2: JoinHandle<Result<()>>,
) -> Result<()> {
    // original donuts
    let mut frames = donuts(); // |  559234 bytes
                               // trim redundant whitespace
    trim_frames(&mut frames); //  |  395012 bytes
                              // results in a 29.37% size reduction

    println!("Listening @ http://{}{}\n", cfg.addr(), cfg.path());

    loop {
        // if either of these threads have finished,
        // end the main thread because an unexpected
        // poison error as ocurred
        if th1.is_finished() {
            break th1.join()?;
        } else if th2.is_finished() {
            break th2.join()?;
        }
        // wait until there is at least one connection
        wait(&streams.1)?;

        // distribute frames to each client
        for frame in frames.iter() {
            // stop distributing if there are no connections
            if !*streams.1 .0.lock()? {
                break;
            }

            //  send each stream the current frame
<<<<<<< HEAD
            for (ip, mut stream) in streams.0.read()?.iter() {
                //  fails if the stream disconnected from the server
                if let Err(_) = stream.write_all(frame) {
                    disconnected.0.write()?.push(*ip);
                    *disconnected.1 .0.lock()? = true;
                    disconnected.1 .1.notify_one()
=======
            for (ip, mut stream) in streams.iter() {
                //  fails if the stream disconnected from the server
                if let Err(_) = stream.write_all(frame) {
                    disconnected.push(*ip)
>>>>>>> e13826e13ffea178880ea30ab20dd6cad2efd513
                }
            }
            sleep(DELAY)
        }
    }
}

fn main() -> Result<()> {
    let cfg = Config::parse();

    //  initiate the listener
    let server = TcpListener::bind(cfg.addr())?;

    let streams: Arc<(RwLock<HashMap<u16, TcpStream>>, Arc<(Mutex<bool>, Condvar)>)> =
        Default::default();
    let disconnected: Arc<(RwLock<Vec<u16>>, Arc<(Mutex<bool>, Condvar)>)> = Default::default();

    let th1 = incoming_handler(server, streams.clone(), cfg.path().to_string());
    let th2 = error_handler(streams.clone(), disconnected.clone());

    main_thread(cfg, streams, disconnected, th1, th2)
}
