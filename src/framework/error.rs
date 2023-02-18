//! Error types and conversion functions.

use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error;
use std::sync::mpsc::SendError;
use std::sync::{Arc, PoisonError};

/// An enum containing all kinds of game framework errors.
#[derive(Debug, Clone)]
pub enum GameError {
    /// An error in the filesystem layout
    FilesystemError(String),
    /// An error in the config file
    ConfigError(String),
    /// Happens when an `winit::EventsLoopProxy` attempts to
    /// wake up an `winit::EventsLoop` that no longer exists.
    EventLoopError(String),
    /// An error trying to load a resource, such as getting an invalid image file.
    ResourceLoadError(String),
    /// Unable to find a resource; the `Vec` is the paths it searched for and associated errors
    ResourceNotFound(String, Vec<(std::path::PathBuf, GameError)>),
    /// Something went wrong in the renderer
    RenderError(String),
    /// Something went wrong in the audio playback
    AudioError(String),
    /// Something went wrong trying to set or get window properties.
    WindowError(String),
    /// Something went wrong trying to read from a file
    IOError(Arc<std::io::Error>),
    /// Something went wrong trying to load/render a font
    FontError(String),
    /// Something went wrong applying video settings.
    VideoError(String),
    /// Something went wrong compiling shaders
    ShaderProgramError(String),
    /// Something went wrong with the `gilrs` gamepad-input library.
    GamepadError(String),
    /// Something went wrong with the `lyon` shape-tesselation library.
    LyonError(String),
    /// Something went wrong while parsing something.
    ParseError(String),
    /// Something went wrong while converting a value.
    InvalidValue(String),
    /// Something went wrong while executing a debug command line command.
    CommandLineError(String),
    /// Something went wrong while initializing or modifying Discord rich presence values.
    DiscordRPCError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameError::ConfigError(ref s) => write!(f, "Config error: {}", s),
            GameError::ResourceLoadError(ref s) => write!(f, "Error loading resource: {}", s),
            GameError::ResourceNotFound(ref s, ref paths) => {
                write!(f, "Resource not found: {}, searched in paths {:?}", s, paths)
            }
            GameError::WindowError(ref e) => write!(f, "Window creation error: {}", e),
            _ => write!(f, "GameError {:?}", self),
        }
    }
}

impl Error for GameError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            GameError::IOError(e) => Some(e as &dyn Error),
            _ => None,
        }
    }
}

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T = ()> = Result<T, GameError>;

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(Arc::new(e))
    }
}

impl From<image::ImageError> for GameError {
    fn from(e: image::ImageError) -> GameError {
        let errstr = format!("Image load error: {}", e);
        GameError::ResourceLoadError(errstr)
    }
}

impl From<std::string::FromUtf8Error> for GameError {
    fn from(e: FromUtf8Error) -> Self {
        let errstr = format!("UTF-8 decoding error: {:?}", e);
        GameError::ConfigError(errstr)
    }
}

impl From<serde_json::Error> for GameError {
    fn from(e: serde_json::Error) -> Self {
        let errstr = format!("JSON error: {:?}", e);
        GameError::ParseError(errstr)
    }
}

#[cfg(target_os = "android")]
impl From<jni::errors::Error> for GameError {
    fn from(e: jni::errors::Error) -> GameError {
        GameError::WindowError(e.to_string())
    }
}

impl From<strum::ParseError> for GameError {
    fn from(s: strum::ParseError) -> GameError {
        let errstr = format!("Strum parse error: {}", s);
        GameError::ParseError(errstr)
    }
}

impl From<cpal::DefaultStreamConfigError> for GameError {
    fn from(s: cpal::DefaultStreamConfigError) -> GameError {
        let errstr = format!("Default stream config error: {}", s);
        GameError::AudioError(errstr)
    }
}

impl From<cpal::PlayStreamError> for GameError {
    fn from(s: cpal::PlayStreamError) -> GameError {
        let errstr = format!("Play stream error: {}", s);
        GameError::AudioError(errstr)
    }
}

impl From<cpal::BuildStreamError> for GameError {
    fn from(s: cpal::BuildStreamError) -> GameError {
        let errstr = format!("Build stream error: {}", s);
        GameError::AudioError(errstr)
    }
}

impl<T> From<PoisonError<T>> for GameError {
    fn from(s: PoisonError<T>) -> GameError {
        let errstr = format!("Poison error: {}", s);
        GameError::EventLoopError(errstr)
    }
}

impl<T> From<SendError<T>> for GameError {
    fn from(s: SendError<T>) -> GameError {
        let errstr = format!("Send error: {}", s);
        GameError::EventLoopError(errstr)
    }
}
