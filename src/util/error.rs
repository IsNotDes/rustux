//! Error handling for RustUX

use thiserror::Error;

/// Main error type for RustUX
#[derive(Error, Debug)]
pub enum Error {
    #[error("SDL2 error: {0}")]
    Sdl2(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Game logic error: {0}")]
    GameLogic(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Video error: {0}")]
    Video(String),

    #[error("Physics error: {0}")]
    Physics(String),

    #[error("Level loading error: {0}")]
    LevelLoading(String),

    #[error("Sprite loading error: {0}")]
    SpriteLoading(String),
#[error("Asset download error: {0}")]
    AssetDownload(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Unknown(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Unknown(s.to_string())
    }
}

/// Result type alias for RustUX
pub type Result<T> = std::result::Result<T, Error>;

/// Helper macro for creating errors
#[macro_export]
macro_rules! rustux_error {
    ($variant:ident, $msg:expr) => {
        $crate::util::Error::$variant($msg.to_string())
    };($variant:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::util::Error::$variant(format!($fmt, $($arg)*))
    };
}

/// Helper macro for creating results
#[macro_export]
macro_rules! rustux_bail {
    ($variant:ident, $msg:expr) => {
        return Err($crate::rustux_error!($variant, $msg))
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        return Err($crate::rustux_error!($variant, $fmt, $($arg)*))
    };
}