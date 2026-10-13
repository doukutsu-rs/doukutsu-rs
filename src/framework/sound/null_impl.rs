use std::any::Any;
use crate::framework::backend::{BackendAudio, AudioEvent};

pub struct NullAudio;

impl BackendAudio for NullAudio {
    fn sample_rate(&self) -> u32 {
        44100
    }

    fn channels(&self) -> u16 {
        2
    }

    fn set_paused(&mut self, _paused: bool) {}

    fn update(&mut self) -> Vec<AudioEvent> {
        Vec::new()
    }

    fn set_callback(&mut self, _callback: Box<dyn FnMut(&mut [f32]) + Send>) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
