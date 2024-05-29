//! Error types and conversion functions.

use core::fmt;
use std::sync::mpsc::SendError;
use std::sync::{Arc, PoisonError};

pub use drs_framework::error::*;

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

impl From<log::SetLoggerError> for GameError {
    fn from(s: log::SetLoggerError) -> GameError {
        let errstr = format!("Set logger error: {}", s);
        GameError::LoggerError(errstr)
    }
}
