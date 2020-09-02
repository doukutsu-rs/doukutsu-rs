#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RiffChunk {
    id: [u8; 4],
    length: u32
}

use std::fmt;

impl fmt::Display for RiffChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::ascii::escape_default as esc;
        
        write!(f, "chunk \"{}{}{}{}\", length: {}",
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
    pub bit_depth: u16
}

impl fmt::Display for WavFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} channels, {} Hz, {}-bit",
            self.channels,
            self.sample_rate,
            self.bit_depth
        )
    }
}

#[derive(Clone)]
pub struct WavSample {
    pub format: WavFormat,
    pub data: Vec<u8>
}

impl fmt::Display for WavSample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {} samples",
            self.format,
            // num_bytes / bytes_per_sample
            self.data.len() / ((self.format.bit_depth / 8) * self.format.channels) as usize
        )
    }
}

use byteorder::{LE, ReadBytesExt};
use std::io;

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
            b"RIFF" => {},
            b"RIFX" => panic!("Cannot handle RIFX data!"),
            _       => panic!("Expected RIFF signature, found {}", riff)
        }
        
        let mut rfmt = [0; 4];
        
        f.read_exact(&mut rfmt)?;
        
        assert_eq!(rfmt, *b"WAVE");
        
        let fmt = RiffChunk::read_from(&mut f)?;
        
        assert_eq!(fmt.id, *b"fmt ");
        //assert_eq!(fmt.length, 16);
        
        let afmt = f.read_u16::<LE>()?;
        
        debug_assert!(afmt == 1);
        
        let channels = f.read_u16::<LE>()?;
        let samples  = f.read_u32::<LE>()?;
        let _brate = f.read_u32::<LE>()?;
        let _balgn = f.read_u16::<LE>()?;
        let bits     = f.read_u16::<LE>()?;
        
        let data = RiffChunk::read_from(&mut f)?;
        
        assert_eq!(data.id, *b"data");
        
        let mut buf = vec![0; data.length as usize];
        
        f.read_exact(&mut buf)?;
        
        Ok(
            WavSample {
                format: WavFormat {
                    channels,
                    sample_rate: samples,
                    bit_depth: bits
                },
                data: buf
            }
        )
    }
}
