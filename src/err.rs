pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum Header {
    Method,
    Path,
    Version,
    UserAgent,
    Accept,
}

impl AsRef<str> for Header {
    fn as_ref(&self) -> &str {
        match self {
            Self::Method => "Method",
            Self::Path => "Path",
            Self::Version => "Version",
            Self::UserAgent => "User-Agent",
            Self::Accept => "Accept",
        }
    }
}

pub enum Invalid {
    HttpParse(httparse::Error),
    UriParse(uriparse::PathError),
    Header(Header),
    DuplicateStream,
    Format,
}

impl ToString for Invalid {
    fn to_string(&self) -> String {
        match self {
            Self::HttpParse(e) => e.to_string(),
            Self::UriParse(e) => e.to_string(),
            Self::Header(h) => format!("Invalid `{}` header value", h.as_ref()),
            Self::DuplicateStream => "Invalid duplicate stream".to_string(),
            Self::Format => "Invalid http format".to_string(),
        }
    }
}

impl From<httparse::Error> for Invalid {
    fn from(value: httparse::Error) -> Self {
        Self::HttpParse(value)
    }
}

impl From<uriparse::PathError> for Invalid {
    fn from(value: uriparse::PathError) -> Self {
        Self::UriParse(value)
    }
}

impl From<Header> for Invalid {
    fn from(value: Header) -> Self {
        Self::Header(value)
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

impl From<Box<dyn std::any::Any + Send + 'static>> for Error {
    fn from(_: Box<dyn std::any::Any + Send + 'static>) -> Self {
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
