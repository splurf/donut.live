use httparse::{Request, EMPTY_HEADER};
use log::debug;
use std::{
    io::Read,
    net::{IpAddr, SocketAddr, TcpStream},
    str::FromStr,
    thread::{spawn, JoinHandle},
};

use super::{Invalid, Result, UriError};

/// Verify the potential client by checking if the User-Agent's product is `curl` and a few other practicalities
pub fn verify_stream(mut stream: &TcpStream, uri_path: &str) -> Result<SocketAddr> {
    // read from the incoming stream
    let mut buf = [0; 128];
    let bytes = stream.read(&mut buf)?;

    // parse the request
    let mut headers = [EMPTY_HEADER; 8];
    let mut req = Request::new(&mut headers);
    _ = req.parse(&buf[..bytes])?;

    // validate the request
    if let (Some(method), Some(path), Some(version)) = (req.method, req.path, req.version) {
        if method != "GET" {
            Err(UriError::Method(method.to_owned()).into())
        } else if path != uri_path {
            Err(UriError::Path(path.to_owned()).into())
        } else if version != 1 {
            Err(UriError::Version(version).into())
        // check for any incompatible headers
        } else if let Some(h) = req.headers.iter().find(|h| match h.name {
            "User-Agent" => !h.value.starts_with(b"curl"),
            "Accept" => h.value != b"*/*",
            _ => false,
        }) {
            Err(UriError::from(h).into())
        } else {
            // retrieve client IP-address and port number
            let peer_addr = stream.peer_addr()?;

            // attempt to parse real remote address, if specified
            if let Some(remote_addr_header) = req.headers.iter().find(|h| h.name == "X-Real-IP") {
                IpAddr::from_str(&String::from_utf8(remote_addr_header.value.to_vec())?)
                    .map(|addr| SocketAddr::new(addr, peer_addr.port()))
                    .map_err(Into::into)
            // otherwise, return original addr
            } else {
                Ok(peer_addr)
            }
        }
    } else {
        Err(Invalid::Format.into())
    }
}

/// Spawn a new thread that repeatedly calls the provided function.
pub fn init_handler(f: impl FnMut() -> Result + Send + 'static) -> JoinHandle<Result> {
    spawn(move || loop_func(f))
}

/// Continuously call the provided function while emitting errors.
pub fn loop_func(mut f: impl FnMut() -> Result + Send + 'static) -> Result {
    loop {
        // call the function
        if let Err(e) = f() {
            debug!("{}", e)
        }
    }
}
