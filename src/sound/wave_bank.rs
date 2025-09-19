// SPDX-License-Identifier: MIT
// Copyright (c) 2020 LunarLambda
// Copyright (c) 2020 doukutsu-rs contributors (see AUTHORS.md)
use std::fmt;
use std::io;
use std::sync::Arc;

use crate::sound::wav;

#[derive(Clone)]
pub struct SoundBank {
    pub wave100: Arc<[u8; 25600]>,
    pub samples: Arc<[wav::WavSample]>,
}

impl fmt::Display for SoundBank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "WAVE100: {:2X?}...", &self.wave100[..8])?;

        for sample in self.samples.iter() {
            writeln!(f, "{}", sample)?;
        }

        Ok(())
    }
}

impl SoundBank {
    pub fn load_from<R: io::Read>(mut f: R) -> io::Result<SoundBank> {
        let mut wave100 = Box::new([0u8; 25600]);

        f.read_exact(wave100.as_mut())?;

        let mut samples = Vec::with_capacity(16);

        loop {
            match wav::WavSample::read_from(&mut f) {
                Ok(sample) => {
                    log::info!("Loaded sample: {:?}", sample.format);
                    samples.push(sample)
                }
                Err(err) => {
                    log::error!("Failed to read next sample: {}", err);
                    return Ok(SoundBank { wave100: wave100.into(), samples: samples.into() });
                }
            }
        }
    }

    pub fn get_wave(&self, index: usize) -> &[u8] {
        &self.wave100[index * 256..(index + 1) * 256]
    }
}
