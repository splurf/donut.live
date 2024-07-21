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

    /// URI location path
    #[arg(long, default_value_t = String::from('/'), value_parser = parse_path)]
    path: String,

    /// Custom provided GIF
    #[arg(short, long)]
    gif: Option<PathBuf>,

    /// Custom Frames/sec
    #[arg(short, long)]
    fps: Option<f32>,

    /// Enable/Disable color
    #[arg(short, long)]
    colored: bool,
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

    /// URI path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Path to the images.
    pub fn gif(&self) -> Option<&Path> {
        self.gif.as_deref()
    }

    /// Frames/second, if specified.
    pub const fn fps(&self) -> Option<f32> {
        self.fps
    }

    /// Determinant for whether the gif will be is_colored or not.
    pub const fn is_colored(&self) -> bool {
        self.colored
    }

    /// The file stem of the ascii-generated file.
    fn file_stem(&self) -> &str {
        self.gif()
            .map(|p| {
                p.file_stem()
                    .map(|s| s.to_str().unwrap_or("_"))
                    .unwrap_or("_")
            })
            .unwrap_or("donuts")
    }

    /// The file name of the ascii-generated file.
    pub fn file_name(&self) -> String {
        // '.ascii' default extension
        let mut s = format!("{}.ascii", self.file_stem());

        // '.asciic' extension indicates colored ascii
        if self.is_colored() {
            s.push('c')
        }
        s
    }
}
