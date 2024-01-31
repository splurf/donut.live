pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum UriError {
    HttpParse(httparse::Error),
    UriParse(uriparse::PathError),
    Method(String),
    Path(String),
    Version(u8),
    Header(String),
}

impl<'a> From<&mut httparse::Header<'a>> for UriError {
    fn from(value: &mut httparse::Header<'a>) -> Self {
        Self::Header(format!(
            "{{ {}: {} }}",
            value.name,
            String::from_utf8_lossy(value.value)
        ))
    }
}

impl ToString for UriError {
    fn to_string(&self) -> String {
        match self {
            Self::HttpParse(e) => e.to_string(),
            Self::UriParse(e) => e.to_string(),
            Self::Method(s) => format!("method {}", s),
            Self::Path(s) => format!("path {}", s),
            Self::Version(v) => format!("version {}", v),
            Self::Header(s) => format!("header {}", s),
        }
    }
}

pub enum Invalid {
    Uri(UriError),
    Format,
}

impl ToString for Invalid {
    fn to_string(&self) -> String {
        format!(
            "Invalid {}",
            match self {
                Self::Uri(e) => e.to_string(),
                Self::Format => "http format".to_string(),
            }
        )
    }
}

impl From<httparse::Error> for Invalid {
    fn from(value: httparse::Error) -> Self {
        UriError::HttpParse(value).into()
    }
}

impl From<uriparse::PathError> for Invalid {
    fn from(value: uriparse::PathError) -> Self {
        UriError::UriParse(value).into()
    }
}

impl From<UriError> for Invalid {
    fn from(value: UriError) -> Self {
        Self::Uri(value)
    }
}

pub enum Error {
    IO(std::io::Error),
    Parse(Invalid),
    Sync,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl<T: Into<Invalid>> From<T> for Error {
    fn from(value: T) -> Self {
        Self::Parse(value.into())
    }
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        Self::Sync
    }
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>) -> Self {
        Self::Sync
    }
}

impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        Self::Sync
    }
}

impl From<Box<dyn std::any::Any + Send>> for Error {
    fn from(_: Box<dyn std::any::Any + Send>) -> Self {
        Self::Sync
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::IO(e) => e.to_string(),
            Self::Parse(e) => e.to_string(),
            Self::Sync => "An unexpected poison error has ocurred".to_string(),
        })
    }
}

impl std::error::Error for Error {}
