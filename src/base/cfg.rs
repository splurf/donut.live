use clap::Parser;
use log::Level;
use std::{
    env::{set_var, var},
    net::{IpAddr, Ipv4Addr, SocketAddr},
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
    #[arg(long)]
    fps: Option<f32>,

    /// Enable/Disable color
    #[arg(short, long)]
    colored: bool,

    /// Ensure 'COLORTERM' and 'CLICOLOR_FORCE' are set
    #[arg(short, long)]
    force_colored: bool,
}

impl InitConfig {
    /// Construct a [`SocketAddr`] from the `addr` and `port` attributes
    pub const fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.addr, self.port)
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
        self.colored || self.force_colored
    }
}

pub struct Config {
    init: InitConfig,
    file_name: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        // ensure valid log level (default: 'trace')
        let log_level = Level::from_str(&var("RUST_LOG").unwrap_or_else(|_| "trace".to_string()))?;

        // have 'clap' parse the program arguments
        let init = InitConfig::parse();

        // ensure intended log level
        set_var("RUST_LOG", format!("{},artem=warn", log_level));

        // set color specifiers
        if init.force_colored {
            set_var("COLORTERM", "truecolor");
            set_var("CLICOLOR_FORCE", "1");
        }

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

        Ok(Self { init, file_name })
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

impl Deref for Config {
    type Target = InitConfig;

    fn deref(&self) -> &Self::Target {
        &self.init
    }
}
