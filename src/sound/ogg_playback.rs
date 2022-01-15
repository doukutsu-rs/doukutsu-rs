use std::sync::{Arc, RwLock};

use lewton::inside_ogg::OggStreamReader;
use num_traits::clamp;

use crate::framework::filesystem::File;
use crate::sound::wav::WavFormat;

pub(crate) struct OggPlaybackEngine {
    intro_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    loop_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    output_format: WavFormat,
    playing_intro: bool,
    position: u64,
    buffer: Vec<i16>,
}

pub struct SavedOggPlaybackState {
    intro_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    loop_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    playing_intro: bool,
    position: u64,
}

impl OggPlaybackEngine {
    pub fn new() -> OggPlaybackEngine {
        OggPlaybackEngine {
            intro_music: None,
            loop_music: None,
            output_format: WavFormat {
                channels: 2,
                sample_rate: 44100,
                bit_depth: 16,
            },
            playing_intro: false,
            position: 0,
            buffer: Vec::with_capacity(4096),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: usize) {
        self.output_format.sample_rate = sample_rate as u32;
    }

    pub fn get_state(&self) -> SavedOggPlaybackState {
        SavedOggPlaybackState {
            intro_music: self.intro_music.clone(),
            loop_music: self.loop_music.clone(),
            playing_intro: self.playing_intro,
            position: self.position,
        }
    }

    pub fn set_state(&mut self, state: SavedOggPlaybackState) {
        self.intro_music = state.intro_music;
        self.loop_music = state.loop_music;
        self.playing_intro = state.playing_intro;
        self.position = state.position;
    }

    pub fn start_single(&mut self, loop_music: Box<OggStreamReader<File>>) {
        self.intro_music = None;
        self.loop_music = Some(Arc::new(RwLock::new(loop_music)));
        self.playing_intro = false;
        self.position = 0;
    }

    pub fn start_multi(
        &mut self,
        intro_music: Box<OggStreamReader<File>>,
        loop_music: Box<OggStreamReader<File>>,
    ) {
        self.intro_music = Some(Arc::new(RwLock::new(intro_music)));
        self.loop_music = Some(Arc::new(RwLock::new(loop_music)));
        self.playing_intro = true;
        self.position = 0;
    }

    pub fn rewind(&mut self) {
        if let Some(music) = self.intro_music.as_ref() {
            let _ = music.write().unwrap().seek_absgp_pg(0);
            self.position = 0;
            self.playing_intro = true;
        } else {
            if let Some(music) = self.loop_music.as_ref() {
                let _ = music.write().unwrap().seek_absgp_pg(0);
            }

            self.position = 0;
            self.playing_intro = false;
        }
    }

    fn decode(&mut self) {
        if self.playing_intro {
            if let Some(music) = self.intro_music.as_ref() {
                let mut music = music.write().unwrap();

                let mut buf = match music.read_dec_packet_itl() {
                    Ok(Some(buf)) => buf,
                    Ok(None) => {
                        self.playing_intro = false;
                        return;
                    }
                    Err(e) => {
                        log::error!("Error decoding intro: {}", e);
                        self.playing_intro = false;
                        return;
                    }
                };

                self.position = music.get_last_absgp().unwrap_or(0);
                buf = self.resample_buffer(
                    buf,
                    music.ident_hdr.audio_sample_rate,
                    music.ident_hdr.audio_channels,
                );
                self.buffer.append(&mut buf);
            } else {
                self.playing_intro = false;
            }
        } else {
            if let Some(music) = self.loop_music.as_ref() {
                let mut music = music.write().unwrap();

                let mut buf = match music.read_dec_packet_itl() {
                    Ok(Some(buf)) => buf,
                    Ok(None) => {
                        if let Err(_) = music.seek_absgp_pg(0) {
                            vec![0, 1000]
                        } else {
                            return;
                        }
                    }
                    Err(_) => {
                        vec![0, 1000]
                    }
                };

                self.position = music.get_last_absgp().unwrap_or(0);
                buf = self.resample_buffer(
                    buf,
                    music.ident_hdr.audio_sample_rate,
                    music.ident_hdr.audio_channels,
                );
                self.buffer.append(&mut buf);
            } else {
                let mut buf = vec![0; 1000];
                self.buffer.append(&mut buf);
            }
        }
    }

    fn resample_buffer(&self, mut data: Vec<i16>, sample_rate: u32, channels: u8) -> Vec<i16> {
        if channels == 1 {
            let mut tmp_data = Vec::with_capacity(data.len() * 2);

            for s in data.iter().copied() {
                tmp_data.push(s);
                tmp_data.push(s);
            }

            data = tmp_data;
        }

        if sample_rate != self.output_format.sample_rate {
            let mut tmp_data = Vec::with_capacity((data.len() as f64 * self.output_format.sample_rate as f64 / sample_rate as f64) as usize);
            let mut pos = 0.0;
            let phase = sample_rate as f32 / self.output_format.sample_rate as f32;

            loop {
                if pos >= data.len() as f32 {
                    data = tmp_data;
                    break;
                }

                let s = unsafe {
                    let upos = pos as usize;
                    let s1 = (*data.get_unchecked(upos) as f32) / 32768.0;
                    let s2 =
                        (*data.get_unchecked(clamp(upos + 1, 0, data.len() - 1)) as f32) / 32768.0;

                    ((s1 + (s2 - s1) * pos.fract()) * 32768.0) as i16
                };
                tmp_data.push(s);

                pos += phase;
            }
        }

        data
    }

    pub fn render_to(&mut self, buf: &mut [u16]) -> usize {
        while self.buffer.len() < buf.len() {
            self.decode();
        }

        self.buffer
            .drain(0..buf.len())
            .map(|n| n as u16 ^ 0x8000)
            .zip(buf.iter_mut())
            .for_each(|(n, tgt)| *tgt = n);

        buf.len()
    }
}
