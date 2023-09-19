use {
    super::err::Result,
    clap::Parser,
    std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    uriparse::Path,
};

fn validate_path(path: &str) -> Result<String> {
    let mut path = Path::try_from(path)?.to_string();
    if !path.starts_with('/') {
        path.insert(0, '/')
    }
    Ok(path)
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
    addr: IpAddr,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value_t = String::from("/"), value_parser = validate_path)]
    path: String,
}

impl Config {
    pub const fn addr(&self) -> SocketAddr {
        match self.addr {
            IpAddr::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip, self.port)),
            IpAddr::V6(ip) => SocketAddr::V6(SocketAddrV6::new(ip, self.port, 0, 1)),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
