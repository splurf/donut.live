pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum UriError {
    HttpParse(httparse::Error),
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

impl std::fmt::Display for UriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::HttpParse(e) => e.to_string(),
            Self::Method(s) => format!("method {}", s),
            Self::Path(s) => format!("path {}", s),
            Self::Version(v) => format!("version {}", v),
            Self::Header(s) => format!("header {}", s),
        })
    }
}

pub enum GifError {
    Delay,
}

impl std::fmt::Display for GifError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Delay => "GIF (missing frame rate)",
        })
    }
}

impl From<GifError> for Invalid {
    fn from(value: GifError) -> Self {
        Self::Gif(value)
    }
}

pub enum Invalid {
    Uri(UriError),
    Gif(GifError),
    Format,
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Invalid {}",
            match self {
                Self::Uri(e) => e.to_string(),
                Self::Gif(e) => e.to_string(),
                Self::Format => "http format".to_string(),
            }
        ))
    }
}

impl From<httparse::Error> for Invalid {
    fn from(value: httparse::Error) -> Self {
        UriError::HttpParse(value).into()
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
    Gif(image::ImageError),
    Json(bincode::Error),
    Empty,
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

impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::Gif(value)
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

impl From<bincode::Error> for Error {
    fn from(value: bincode::Error) -> Self {
        Self::Json(value)
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
            Self::Gif(e) => e.to_string(),
            Self::Json(e) => e.to_string(),
            Self::Empty => "The server is empty. Entering idle mode.".to_string(),
            Self::Sync => "An unexpected poison error has occurred".to_string(),
        })
    }
}

impl std::error::Error for Error {}
