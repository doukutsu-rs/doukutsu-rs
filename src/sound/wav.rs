use core::fmt;

use drs_framework::{
    error::{GameError, GameResult},
    io,
};

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
    pub data: Vec<u8>,
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
    pub fn read_from<R: io::Read>(mut f: R) -> GameResult<RiffChunk> {
        let mut id = [0; 4];

        f.read_exact(&mut id)?;
        let length = f.read_u32_le()?;

        Ok(RiffChunk { id, length })
    }
}

impl WavSample {
    pub fn read_from<R: io::Read>(mut f: R) -> GameResult<WavSample> {
        let riff = RiffChunk::read_from(f.by_ref())?;

        match &riff.id {
            b"RIFF" => {}
            b"RIFX" => return Err(GameError::ParseError("Cannot handle RIFX data!".to_owned())),
            _ => {
                return Err(GameError::ParseError(format!("Expected RIFF signature, found {}", riff)));
            }
        }

        let mut rfmt = [0; 4];

        f.read_exact(&mut rfmt)?;

        if rfmt != *b"WAVE" {
            return Err(GameError::ParseError("Expected 'WAVE' RIFF chunk.".to_owned()));
        }

        let fmt = RiffChunk::read_from(f.by_ref())?;

        if fmt.id != *b"fmt " {
            return Err(GameError::ParseError("Expected 'fmt ' RIFF chunk.".to_owned()));
        }

        let afmt = f.read_u16_le()?;

        if afmt != 1 {
            return Err(GameError::ParseError("Only PCM audio data is supported.".to_owned()));
        }

        let channels = f.read_u16_le()?;
        let samples = f.read_u32_le()?;
        let _brate = f.read_u32_le()?;
        let _balgn = f.read_u16_le()?;
        let bits = f.read_u16_le()?;

        let data = RiffChunk::read_from(f.by_ref())?;

        if data.id != *b"data" {
            return Err(GameError::ParseError("Expected 'data' RIFF chunk.".to_owned()));
        }

        let mut buf = vec![0; data.length as usize];

        f.read_exact(&mut buf)?;

        Ok(WavSample { format: WavFormat { channels, sample_rate: samples, bit_depth: bits }, data: buf })
    }
}
