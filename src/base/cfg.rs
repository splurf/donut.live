use clap::Parser;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::{Path, PathBuf},
};

use super::Result;

/// Parse the provided path, ensuring it has a root.
fn parse_path(s: &str) -> Result<String> {
    let mut p = s.to_owned();
    if !p.starts_with('/') {
        p.insert(0, '/')
    }
    Ok(p)
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    /// IP address
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
    addr: IpAddr,

    /// Port number
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Location path
    #[arg(long, default_value_t = String::from('/'), value_parser = parse_path)]
    path: String,

    #[arg(short, long)]
    images: Option<PathBuf>,

    #[arg(short, long)]
    fps: Option<f32>,
}

impl Config {
    pub fn new() -> Self {
        Self::parse()
    }

    /// Construct a [`SocketAddr`] from the `addr` and `port` attributes
    pub const fn addr(&self) -> SocketAddr {
        match self.addr {
            IpAddr::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip, self.port)),
            IpAddr::V6(ip) => SocketAddr::V6(SocketAddrV6::new(ip, self.port, 0, 1)),
        }
    }

    /// Return the URI path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Return the path to the images.
    pub fn images(&self) -> Option<&Path> {
        self.images.as_deref()
    }

    /// Return the frames/second, if specified.
    pub const fn fps(&self) -> Option<f32> {
        self.fps
    }
}
