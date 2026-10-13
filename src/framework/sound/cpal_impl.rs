use std::any::Any;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::framework::backend::{BackendAudio, AudioEvent};
use crate::framework::error::{GameError, GameResult};

pub struct CpalAudio {
    stream: Option<cpal::Stream>,
    sample_rate: u32,
    channels: u16,
    paused: bool,
    error_rx: Receiver<AudioEvent>,
    error_tx: Sender<AudioEvent>,
    callback: Option<Arc<Mutex<Box<dyn FnMut(&mut [f32]) + Send>>>>,
}

impl CpalAudio {
    pub fn new() -> GameResult<Box<dyn BackendAudio>> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| GameError::AudioError("No output device found".to_string()))?;

        let config = device.default_output_config()
            .map_err(|e| GameError::AudioError(e.to_string()))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let (error_tx, error_rx) = mpsc::channel();

        Ok(Box::new(Self {
            stream: None,
            sample_rate,
            channels,
            paused: false,
            error_rx,
            error_tx,
            callback: None,
        }))
    }

    fn create_stream<T>(&self, device: &cpal::Device, config: &cpal::StreamConfig, callback: Arc<Mutex<Box<dyn FnMut(&mut [f32]) + Send>>>) -> GameResult<cpal::Stream>
    where T: cpal::SizedSample + cpal::FromSample<f32>
    {
        let error_tx = self.error_tx.clone();
        let mut f32_buf = Vec::new();
        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                if f32_buf.len() != data.len() {
                    f32_buf.resize(data.len(), 0.0);
                }

                if let Ok(mut cb) = callback.lock() {
                    cb(&mut f32_buf);
                }

                for (o, i) in data.iter_mut().zip(f32_buf.iter()) {
                    *o = T::from_sample(*i);
                }
            },
            move |err| {
                log::error!("CPAL stream error: {}", err);
                let _ = error_tx.send(AudioEvent::DeviceLost);
            },
            None
        ).map_err(|e| GameError::AudioError(e.to_string()))?;

        Ok(stream)
    }
}

impl BackendAudio for CpalAudio {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if let Some(stream) = &self.stream {
            if paused {
                let _ = stream.pause();
            } else {
                let _ = stream.play();
            }
        }
    }

    fn update(&mut self) -> Vec<AudioEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.error_rx.try_recv() {
            events.push(event);
        }
        events
    }

    fn set_callback(&mut self, callback: Box<dyn FnMut(&mut [f32]) + Send>) {
        let callback = Arc::new(Mutex::new(callback));
        self.callback = Some(callback.clone());

        let host = cpal::default_host();
        if let Some(device) = host.default_output_device() {
            if let Ok(config) = device.default_output_config() {
                self.sample_rate = config.sample_rate().0;
                self.channels = config.channels();
                let sample_format = config.sample_format();
                let stream_config: cpal::StreamConfig = config.into();

                let stream = match sample_format {
                    cpal::SampleFormat::F32 => self.create_stream::<f32>(&device, &stream_config, callback),
                    cpal::SampleFormat::I16 => self.create_stream::<i16>(&device, &stream_config, callback),
                    cpal::SampleFormat::U16 => self.create_stream::<u16>(&device, &stream_config, callback),
                    _ => Err(GameError::AudioError("Unsupported sample format".to_string())),
                };

                if let Ok(stream) = stream {
                    let _ = stream.play();
                    self.stream = Some(stream);
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
