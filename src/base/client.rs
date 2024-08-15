use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
};

pub type Clients = HashMap<SocketAddr, Client>;

#[derive(Debug)]
pub struct Client {
    inner: TcpStream,
    addr: SocketAddr,
}

impl Client {
    pub const fn new(inner: TcpStream, addr: SocketAddr) -> Self {
        Self { inner, addr }
    }

    pub const fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl std::io::Write for Client {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl std::io::Write for &Client {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (&self.inner).write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (&self.inner).flush()
    }
}
