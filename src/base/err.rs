pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum AddrError {
    Parse(std::net::AddrParseError),
    Unexpected(std::string::FromUtf8Error),
}

impl From<std::net::AddrParseError> for AddrError {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<std::string::FromUtf8Error> for AddrError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Unexpected(value)
    }
}

impl std::fmt::Display for AddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Parse(e) => e.to_string(),
            Self::Unexpected(e) => e.to_string(),
        })
    }
}

pub enum UriError {
    HttpParse(httparse::Error),
    Method(String),
    Path(String),
    Version(u8),
    Header(String),
    Addr(AddrError),
}

impl<'a> From<&httparse::Header<'a>> for UriError {
    fn from(value: &httparse::Header<'a>) -> Self {
        Self::Header(format!(
            "{{ {}: {} }}",
            value.name,
            String::from_utf8_lossy(value.value)
        ))
    }
}

impl From<httparse::Error> for UriError {
    fn from(value: httparse::Error) -> Self {
        Self::HttpParse(value)
    }
}

impl<T: Into<AddrError>> From<T> for UriError {
    fn from(value: T) -> Self {
        Self::Addr(value.into())
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
            Self::Addr(e) => format!("address => {}", e),
        })
    }
}

pub enum Invalid {
    Uri(UriError),
    Format,
    Level,
}

impl<T: Into<UriError>> From<T> for Invalid {
    fn from(value: T) -> Self {
        Self::Uri(value.into())
    }
}

impl From<log::ParseLevelError> for Invalid {
    fn from(_: log::ParseLevelError) -> Self {
        Invalid::Level
    }
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Invalid {}",
            match self {
                Self::Uri(e) => e.to_string(),
                Self::Format => "http format".to_string(),
                Self::Level => "log level".to_string(),
            }
        ))
    }
}

pub enum GifError {
    Gif(gif::DecodingError),
    Image(image::ImageError),
    Delay,
    Eof,
}

impl std::fmt::Display for GifError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Gif(e) => e.to_string(),
            Self::Image(e) => e.to_string(),
            Self::Delay => "GIF (missing frame rate)".to_string(),
            Self::Eof => "EOF".to_string(),
        })
    }
}

pub enum Error {
    IO(std::io::Error),
    Parse(Invalid),
    Gif(GifError),
    Json(bincode::Error),
    Cli(indicatif::style::TemplateError),
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

impl From<gif::DecodingError> for Error {
    fn from(value: gif::DecodingError) -> Self {
        Self::Gif(GifError::Gif(value))
    }
}

impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::Gif(GifError::Image(value))
    }
}

impl From<GifError> for Error {
    fn from(value: GifError) -> Self {
        Self::Gif(value)
    }
}

impl From<bincode::Error> for Error {
    fn from(value: bincode::Error) -> Self {
        Self::Json(value)
    }
}

impl From<indicatif::style::TemplateError> for Error {
    fn from(value: indicatif::style::TemplateError) -> Self {
        Self::Cli(value)
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
            Self::Cli(e) => e.to_string(),
            Self::Empty => "The server is empty. Entering idle mode.".to_string(),
            Self::Sync => "An unexpected (poison or thread) error has occurred".to_string(),
        })
    }
}

impl std::error::Error for Error {}
