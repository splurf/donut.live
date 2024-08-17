use chrono::Utc;
use log::debug;
use std::{fs::File, io::Write, net::SocketAddr};

use super::Result;

pub fn init_log_file() {
    _ = File::create_new("log")
}

pub fn log_to_file(addr: SocketAddr) -> Result {
    let mut file = File::options().append(true).open("log")?;
    file.write_fmt(format_args!("{},{} ", Utc::now().to_rfc3339(), addr))?;
    file.flush()?;

    debug!("'{}' logged to file", addr);
    Ok(())
}
