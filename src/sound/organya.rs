use std::io;

use byteorder::{LE, ReadBytesExt};

use crate::framework::error::{GameError, GameResult};

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Version {
    // Can't find any files with this signature,
    // But apparently these files had no Pi flag.
    Beta = b'1',
    Main = b'2',
    // OrgMaker 2.05 Extended Drums
    Extended = b'3',
}

#[derive(Debug, Copy, Clone)]
pub struct LoopRange {
    // inclusive
    pub start: i32,
    // exclusive
    pub end: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Display {
    pub beats: u8,
    pub steps: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Timing {
    pub wait: u16,
    pub loop_range: LoopRange,
}

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Instrument {
    pub freq: u16,
    pub inst: u8,
    pub pipi: u8,
    pub notes: u16,
}

#[derive(Clone)]
pub struct Track {
    pub inst: Instrument,
    pub notes: Vec<Note>,
}

impl std::fmt::Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inst.fmt(f)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Note {
    pub pos: i32,
    pub key: u8,
    pub len: u8,
    pub vol: u8,
    pub pan: u8,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub version: Version,
    pub time: Timing,
    pub tracks: [Track; 16],
}

impl Song {
    pub fn empty() -> Song {
        Song {
            version: Version::Main,
            time: Timing { wait: 8, loop_range: LoopRange { start: 0, end: 1 } },
            tracks: [
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
                Track { inst: Instrument { freq: 1000, inst: 0, pipi: 0, notes: 0 }, notes: vec![] },
            ],
        }
    }

    pub fn load_from<R: io::Read>(mut f: R) -> GameResult<Song> {
        let mut magic = [0; 6];

        f.read_exact(&mut magic)?;

        let version =
            match &magic {
                b"Org-01" => Version::Beta,
                b"Org-02" => Version::Main,
                b"Org-03" => Version::Extended,
                _ => return Err(GameError::ResourceLoadError("Invalid magic number".to_string()))
            };

        let wait = f.read_u16::<LE>()?;
        let _bpm = f.read_u8()?;
        let _spb = f.read_u8()?;
        let start = f.read_i32::<LE>()?;
        let end = f.read_i32::<LE>()?;

        use std::mem::MaybeUninit as Mu;

        let mut insts: [Mu<Instrument>; 16] = unsafe {
            Mu::uninit().assume_init()
        };

        for i in &mut insts {
            let freq = f.read_u16::<LE>()?;
            let inst = f.read_u8()?;
            let pipi = f.read_u8()?;
            let notes = f.read_u16::<LE>()?;

            *i = Mu::new(Instrument {
                freq,
                inst,
                pipi,
                notes,
            });
        }

        let insts: [Instrument; 16] = unsafe {
            std::mem::transmute(insts)
        };

        let mut tracks: [Mu<Track>; 16] = unsafe {
            Mu::uninit().assume_init()
        };

        for (i, t) in tracks.iter_mut().enumerate() {
            let count = insts[i].notes as usize;

            #[repr(C)]
            #[derive(Copy, Clone)]
            struct UninitNote {
                pos: Mu<i32>,
                key: Mu<u8>,
                len: Mu<u8>,
                vol: Mu<u8>,
                pan: Mu<u8>,
            }

            let mut notes: Vec<UninitNote> = unsafe {
                vec![Mu::uninit().assume_init(); count]
            };

            for note in &mut notes {
                note.pos = Mu::new(f.read_i32::<LE>()?);
            }

            for note in &mut notes {
                note.key = Mu::new(f.read_u8()?);
            }

            for note in &mut notes {
                note.len = Mu::new(f.read_u8()?);
            }

            for note in &mut notes {
                note.vol = Mu::new(f.read_u8()?);
            }

            for note in &mut notes {
                note.pan = Mu::new(f.read_u8()?);
            }

            *t = Mu::new(Track {
                inst: insts[i],
                notes: unsafe { std::mem::transmute(notes) },
            });
        }

        let tracks = unsafe {
            std::mem::transmute(tracks)
        };

        let song = Song {
            version,
            time: Timing {
                wait,
                loop_range: LoopRange {
                    start,
                    end,
                },
            },
            tracks,
        };

        Ok(song)
    }
}
