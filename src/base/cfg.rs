use clap::Parser;
use log::Level;
use std::{
    env::{set_var, var},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
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
pub struct InitConfig {
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

impl InitConfig {
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
}

pub struct Config {
    init: InitConfig,
    file_name: String,
    log_level: Level,
}

impl Config {
    pub fn new() -> Result<Self> {
        let init = InitConfig::parse();

        // ensure valid log level (default: 'info')
        let log_level = Level::from_str(&var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))?;

        // ensure intended log level
        set_var("RUST_LOG", format!("{},artem=warn", log_level));

        // the new file stem of the ascii-generated file
        let file_stem = init
            .gif()
            .map(|p| {
                p.file_stem()
                    .map(|s| s.to_str().unwrap_or("_"))
                    .unwrap_or("_")
            })
            .unwrap_or("donuts");

        // the new file name of the ascii-generated file
        let file_name = {
            // '.ascii' default extension
            let mut s = format!("{}.ascii", file_stem);

            // '.asciic' extension indicates colored ascii
            if init.is_colored() {
                s.push('c')
            }
            s
        };

        // init logger
        env_logger::init();

        Ok(Self {
            init,
            file_name,
            log_level,
        })
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub const fn log_level(&self) -> Level {
        self.log_level
    }
}

impl Deref for Config {
    type Target = InitConfig;

    fn deref(&self) -> &Self::Target {
        &self.init
    }
}
