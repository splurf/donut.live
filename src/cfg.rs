use {
    crate::Result,
    clap::Parser,
    std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    uriparse::Path,
};

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
    #[arg(long, default_value_t = String::from("/"), value_parser = |path: &str| -> Result<String> {
        let mut p = Path::try_from(path)?.to_string();
        if !p.starts_with('/') {
            p.insert(0, '/');
        }
        Ok(p)
    })]
    path: String,
}

impl Config {
    /// Construct a [`SocketAddr`] from the `addr` and `port` attributes
    pub const fn addr(&self) -> SocketAddr {
        match self.addr {
            IpAddr::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip, self.port)),
            IpAddr::V6(ip) => SocketAddr::V6(SocketAddrV6::new(ip, self.port, 0, 1)),
        }
    }

    /// Return the `path` of the address
    pub fn path(&self) -> &str {
        &self.path
    }
}
