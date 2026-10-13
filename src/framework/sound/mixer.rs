use std::sync::{Arc, Mutex};

pub trait AudioSource: Send {
    /// Generate audio data into the buffer.
    /// The buffer is interleaved samples (L, R, L, R, ...).
    fn generate(&mut self, buffer: &mut [f32], sample_rate: u32);
}

pub struct SoftwareMixer {
    sources: Vec<Box<dyn AudioSource>>,
}

impl SoftwareMixer {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: Box<dyn AudioSource>) {
        self.sources.push(source);
    }

    pub fn mix(&mut self, buffer: &mut [f32], sample_rate: u32) {
        buffer.fill(0.0);
        
        let mut temp_buffer = vec![0.0f32; buffer.len()];
        
        for source in &mut self.sources {
            temp_buffer.fill(0.0);
            source.generate(&mut temp_buffer, sample_rate);
            
            for (out, src) in buffer.iter_mut().zip(temp_buffer.iter()) {
                *out += *src;
            }
        }
        
        // Final clamp
        for sample in buffer.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}
