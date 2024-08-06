use httparse::{Request, EMPTY_HEADER};
use log::error;
use std::{
    io::Read,
    net::TcpStream,
    thread::{spawn, JoinHandle},
};

use super::{Invalid, Result, UriError};

/// Verify the potential client by checking if the User-Agent's product is `curl` and a few other practicalities
pub fn verify_stream(mut stream: &TcpStream, uri_path: &str) -> Result<()> {
    // Read from the incoming stream
    let mut buf = [0; 128];
    let bytes = stream.read(&mut buf)?;

    // Parse the request
    let mut headers = [EMPTY_HEADER; 8];
    let mut req = Request::new(&mut headers);
    _ = req.parse(&buf[..bytes])?;

    // Validate the request
    if let (Some(method), Some(path), Some(version)) = (req.method, req.path, req.version) {
        if method != "GET" {
            Err(UriError::Method(method.to_owned()).into())
        } else if path != uri_path {
            Err(UriError::Path(path.to_owned()).into())
        } else if version != 1 {
            Err(UriError::Version(version).into())
        } else if let Some(h) = req
            .headers
            .iter_mut()
            .take_while(|h| !h.name.is_empty())
            .filter_map(|h| match h.name {
                "User-Agent" => (!h.value.starts_with(b"curl")).then_some(h),
                "Accept" => (h.value != b"*/*").then_some(h),
                _ => None,
            })
            .next()
        {
            Err(UriError::from(h).into())
        } else {
            Ok(())
        }
    } else {
        Err(Invalid::Format.into())
    }
}

/// Spawn a new thread that repeatedly calls the provided function.
pub fn init_handler(f: impl FnMut() -> Result<()> + Send + 'static) -> JoinHandle<Result<()>> {
    spawn(move || loop_func(f))
}

/// Continuously call the provided function while emitting errors.
pub fn loop_func(mut f: impl FnMut() -> Result<()> + Send + 'static) -> Result<()> {
    loop {
        // call the function
        if let Err(e) = f() {
            error!("{}", e)
        }
    }
}
