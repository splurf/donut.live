use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, usize>>> for Error {
    fn from(value: PoisonError<MutexGuard<'_, usize>>) -> Self {
        Self(value.to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, HashMap<IpAddr, Arc<Mutex<usize>>>>>> for Error {
    fn from(value: PoisonError<MutexGuard<'_, HashMap<IpAddr, Arc<Mutex<usize>>>>>) -> Self {
        Self(value.to_string())
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self(value.to_string())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error: {}", self.0))
    }
}
