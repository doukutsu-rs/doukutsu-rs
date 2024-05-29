use core::fmt;

use alloc::{
    format,
    string::{FromUtf8Error, String},
};

/// An enum containing all kinds of game framework errors.
#[derive(Debug, Clone)]
pub enum GameError {
    /// An error in the filesystem layout
    FilesystemError(String),
    /// An error in the config file
    ConfigError(String),
    /// An error in the event loop
    EventLoopError(String),
    /// An error trying to load a resource, such as getting an invalid image file
    ResourceLoadError(String),
    /// Unable to find a resource
    ResourceNotFound(String),
    /// Something went wrong in the renderer
    RenderError(String),
    /// Something went wrong in the audio playback
    AudioError(String),
    /// Something went wrong trying to set or get window properties
    WindowError(String),
    /// Something went wrong trying to read from a file
    IOError(IOErrorKind),
    /// Something went wrong trying to load/render a font
    FontError(String),
    /// Something went wrong applying video settings
    VideoError(String),
    /// Something went wrong compiling shaders
    ShaderProgramError(String),
    /// Something went wrong while parsing something
    ParseError(String),
    /// Something went wrong while converting a value
    InvalidValue(String),
    /// Something went wrong while executing a debug command line command
    CommandLineError(String),
    /// Something went wrong while initializing logger
    LoggerError(String),
    /// We ran out of memory
    AllocationError,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameError::ConfigError(ref s) => write!(f, "Config error: {}", s),
            GameError::ResourceLoadError(ref s) => write!(f, "Error loading resource: {}", s),
            GameError::ResourceNotFound(ref s) => {
                write!(f, "Resource not found: {}", s)
            }
            GameError::WindowError(ref e) => write!(f, "Window creation error: {}", e),
            _ => write!(f, "GameError {:?}", self),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GameError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IOErrorKind {
    UnexpectedEof,
    InvalidUtf8Data,
    WriteZero,
    PermissionDenied,
    InvalidInput,
    Unknown,
}

impl fmt::Display for IOErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IOErrorKind::UnexpectedEof => write!(f, "Unexpected EOF"),
            IOErrorKind::InvalidUtf8Data => write!(f, "Invalid UTF-8 data"),
            IOErrorKind::WriteZero => write!(f, "No bytes written"),
            IOErrorKind::PermissionDenied => write!(f, "Permission denied"),
            IOErrorKind::InvalidInput => write!(f, "Invalid input"),
            IOErrorKind::Unknown => write!(f, "Unknown IO error"),
        }
    }
}

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T = ()> = Result<T, GameError>;

impl From<alloc::collections::TryReserveError> for GameError {
    fn from(_: alloc::collections::TryReserveError) -> GameError {
        GameError::AllocationError
    }
}

impl From<alloc::string::FromUtf8Error> for GameError {
    fn from(e: FromUtf8Error) -> Self {
        let errstr = format!("UTF-8 decoding error: {:?}", e);
        GameError::ConfigError(errstr)
    }
}
