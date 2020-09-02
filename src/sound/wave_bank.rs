use crate::sound::wav;

use std::io;
use std::fmt;

pub struct SoundBank {
    // FIXME: would prefer Box<[u8; 25600]>
    pub wave100: Box<[u8]>,
    
    pub samples: Vec<wav::WavSample>
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
        // no box [0; 25600] yet
        let mut wave100 = vec![0; 25600].into_boxed_slice();
        
        f.read_exact(&mut *wave100)?;
        
        let mut samples = Vec::with_capacity(16);
        
        loop {
            match wav::WavSample::read_from(&mut f) {
                Ok(sample) => samples.push(sample),
                Err(_)     => return Ok(SoundBank { wave100, samples })
            }
        }
    }
    
    pub fn get_wave(&self, index: usize) -> &[u8] {
        &self.wave100[index*256..(index+1)*256]
    }
}
