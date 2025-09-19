// SPDX-License-Identifier: MIT
// Copyright (c) 2020 LunarLambda
// Copyright (c) 2020 doukutsu-rs contributors (see AUTHORS.md)
use std::fmt;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use byteorder::{LE, ReadBytesExt};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RiffChunk {
    id: [u8; 4],
    length: u32,
}

impl fmt::Display for RiffChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::ascii::escape_default as esc;

        write!(
            f,
            "chunk \"{}{}{}{}\", length: {}",
            esc(self.id[0]),
            esc(self.id[1]),
            esc(self.id[2]),
            esc(self.id[3]),
            self.length
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct WavFormat {
    pub channels: u16,
    pub sample_rate: u32,
    pub bit_depth: u16,
}

impl fmt::Display for WavFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} channels, {} Hz, {}-bit", self.channels, self.sample_rate, self.bit_depth)
    }
}

#[derive(Clone)]
pub struct WavSample {
    pub format: WavFormat,
    pub data: Arc<[u8]>,
}

impl fmt::Display for WavSample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {} samples",
            self.format,
            // num_bytes / bytes_per_sample
            self.data.len() / ((self.format.bit_depth / 8) * self.format.channels) as usize
        )
    }
}

impl RiffChunk {
    pub fn read_from<R: io::Read>(mut f: R) -> io::Result<RiffChunk> {
        let mut id = [0; 4];

        f.read_exact(&mut id)?;
        let length = f.read_u32::<LE>()?;

        Ok(RiffChunk { id, length })
    }
}

impl WavSample {
    pub fn read_from<R: io::Read>(mut f: R) -> io::Result<WavSample> {
        let riff = RiffChunk::read_from(&mut f)?;

        match &riff.id {
            b"RIFF" => {}
            b"RIFX" => return Err(io::Error::new(ErrorKind::InvalidData, "Cannot handle RIFX data!".to_owned())),
            _ => {
                return Err(io::Error::new(ErrorKind::InvalidData, format!("Expected RIFF signature, found {}", riff)));
            }
        }

        let mut rfmt = [0; 4];

        f.read_exact(&mut rfmt)?;

        if rfmt != *b"WAVE" {
            return Err(io::Error::new(ErrorKind::InvalidData, "Expected 'WAVE' RIFF chunk.".to_owned()));
        }

        let fmt = RiffChunk::read_from(&mut f)?;

        if fmt.id != *b"fmt " {
            return Err(io::Error::new(ErrorKind::InvalidData, "Expected 'fmt ' RIFF chunk.".to_owned()));
        }

        let afmt = f.read_u16::<LE>()?;

        if afmt != 1 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Only PCM audio data is supported.".to_owned()));
        }

        let channels = f.read_u16::<LE>()?;
        let samples = f.read_u32::<LE>()?;
        let _brate = f.read_u32::<LE>()?;
        let _balgn = f.read_u16::<LE>()?;
        let bits = f.read_u16::<LE>()?;

        let data = RiffChunk::read_from(&mut f)?;

        if data.id != *b"data" {
            return Err(io::Error::new(ErrorKind::InvalidData, "Expected 'data' RIFF chunk.".to_owned()));
        }

        let mut buf = vec![0; data.length as usize];

        f.read_exact(&mut buf)?;

        Ok(WavSample { format: WavFormat { channels, sample_rate: samples, bit_depth: bits }, data: buf.into() })
    }
}
