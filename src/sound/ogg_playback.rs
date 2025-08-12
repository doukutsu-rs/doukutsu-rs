use std::sync::{Arc, RwLock};

use lewton::inside_ogg::OggStreamReader;
use num_traits::clamp;

use crate::bitfield;
use crate::framework::filesystem::File;
use crate::sound::stuff::cubic_interp;
use crate::sound::wav::WavFormat;

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct OggPlaybackFlags(u8);

    pub playing_intro, set_playing_intro: 0;
    pub no_loop, set_no_loop: 1;
}

pub(crate) struct OggPlaybackEngine {
    intro_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    loop_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    output_format: WavFormat,
    flags: OggPlaybackFlags,
    position: u64,
    buffer: Vec<i16>,
}

pub struct SavedOggPlaybackState {
    intro_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    loop_music: Option<Arc<RwLock<Box<OggStreamReader<File>>>>>,
    flags: OggPlaybackFlags,
    position: u64,
}

impl OggPlaybackEngine {
    pub fn new() -> OggPlaybackEngine {
        OggPlaybackEngine {
            intro_music: None,
            loop_music: None,
            output_format: WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 },
            flags: OggPlaybackFlags(0),
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
            flags: self.flags,
            position: self.position,
        }
    }

    pub fn set_state(&mut self, state: SavedOggPlaybackState) {
        self.intro_music = state.intro_music;
        self.loop_music = state.loop_music;
        self.flags = state.flags;
        self.position = state.position;
    }

    pub fn start_single(&mut self, loop_music: Box<OggStreamReader<File>>, no_loop: bool) {
        self.intro_music = None;
        self.loop_music = Some(Arc::new(RwLock::new(loop_music)));
        self.flags.set_playing_intro(false);
        self.position = 0;
        self.flags.set_no_loop(no_loop);
    }

    pub fn start_multi(&mut self, intro_music: Box<OggStreamReader<File>>, loop_music: Box<OggStreamReader<File>>, no_loop: bool) {
        self.intro_music = Some(Arc::new(RwLock::new(intro_music)));
        self.loop_music = Some(Arc::new(RwLock::new(loop_music)));
        self.flags.set_playing_intro(true);
        self.position = 0;
        self.flags.set_no_loop(no_loop);
    }

    pub fn rewind(&mut self) {
        if let Some(music) = &self.intro_music {
            let _ = music.write().unwrap().seek_absgp_pg(0);
            self.position = 0;
            self.flags.set_playing_intro(true);
        } else {
            if let Some(music) = &self.loop_music {
                let _ = music.write().unwrap().seek_absgp_pg(0);
            }

            self.position = 0;
            self.flags.set_playing_intro(false);
        }
    }

    fn decode(&mut self) {
        if self.flags.playing_intro() {
            if let Some(music) = &self.intro_music {
                let mut music = music.write().unwrap();

                let mut buf = match music.read_dec_packet_itl() {
                    Ok(Some(buf)) => buf,
                    Ok(None) => {
                        self.flags.set_playing_intro(false);
                        return;
                    }
                    Err(e) => {
                        log::error!("Error decoding intro: {}", e);
                        self.flags.set_playing_intro(false);
                        return;
                    }
                };

                self.position = music.get_last_absgp().unwrap_or(0);
                buf = self.resample_buffer(buf, music.ident_hdr.audio_sample_rate, music.ident_hdr.audio_channels);
                self.buffer.append(&mut buf);
            } else {
                self.flags.set_playing_intro(false);
            }
        } else if let Some(music) = &self.loop_music {
            let mut music = music.write().unwrap();

            let mut buf = match music.read_dec_packet_itl() {
                Ok(Some(buf)) => buf,
                Ok(None) => {
                    if !self.flags.no_loop() && music.seek_absgp_pg(0).is_ok() {
                        return;
                    }

                    vec![0, 1000]
                }
                Err(_) => {
                    vec![0, 1000]
                }
            };

            self.position = music.get_last_absgp().unwrap_or(0);
            buf = self.resample_buffer(buf, music.ident_hdr.audio_sample_rate, music.ident_hdr.audio_channels);
            self.buffer.append(&mut buf);
        } else {
            let mut buf = vec![0; 1000];
            self.buffer.append(&mut buf);
        }
    }

    fn resample_buffer(&self, mut data: Vec<i16>, sample_rate: u32, channels: u8) -> Vec<i16> {
        if data.is_empty() {
            return data;
        }

        if sample_rate != self.output_format.sample_rate {
            let mut tmp_data = Vec::with_capacity(
                (data.len() as f64 * self.output_format.sample_rate as f64 / sample_rate as f64) as usize,
            );
            let mut pos = 0.0;
            let phase = sample_rate as f32 / self.output_format.sample_rate as f32;
            let num_samples = data.len() / channels as usize;

            if channels == 1 {
                loop {
                    if pos >= num_samples as f32 {
                        data = tmp_data;
                        break;
                    }

                    let s = unsafe {
                        let upos = pos as usize;
                        let s1 = (*data.get_unchecked(upos) as f32) / 32768.0;
                        let s2 = (*data.get_unchecked(clamp(upos + 1, 0, data.len() - 1)) as f32) / 32768.0;
                        let s3 = (*data.get_unchecked(clamp(upos + 2, 0, data.len() - 1)) as f32) / 32768.0;
                        let s4 =
                            (*data.get_unchecked(clamp(upos.saturating_sub(1), 0, data.len() - 1)) as f32) / 32768.0;

                        (cubic_interp(s1, s2, s4, s3, pos.fract()) * 32768.0) as i16
                    };
                    tmp_data.push(s);

                    pos += phase;
                }
            } else if channels == 2 {
                let max_samp = (num_samples - 1) * 2;

                // unrolled for performance reasons
                loop {
                    if pos >= num_samples as f32 {
                        data = tmp_data;
                        break;
                    }

                    let sl = unsafe {
                        let upos = pos as usize;
                        let max = max_samp as usize;
                        let s1 = (*data.get_unchecked(upos * 2) as f32) / 32768.0;
                        let s2 = (*data.get_unchecked(clamp((upos + 1) * 2, 0, max)) as f32) / 32768.0;
                        let s3 = (*data.get_unchecked(clamp((upos + 2) * 2, 0, max)) as f32) / 32768.0;
                        let s4 = (*data.get_unchecked(clamp(upos.saturating_sub(1) * 2, 0, max)) as f32) / 32768.0;

                        (cubic_interp(s1, s2, s4, s3, pos.fract()) * 32768.0) as i16
                    };
                    tmp_data.push(sl);

                    let sr = unsafe {
                        let upos = pos as usize;
                        let max = max_samp as usize + 1;
                        let s1 = (*data.get_unchecked(upos * 2 + 1) as f32) / 32768.0;
                        let s2 = (*data.get_unchecked(clamp((upos + 1) * 2 + 1, 1, max)) as f32) / 32768.0;
                        let s3 = (*data.get_unchecked(clamp((upos + 2) * 2 + 1, 1, max)) as f32) / 32768.0;
                        let s4 = (*data.get_unchecked(clamp(upos.saturating_sub(1) * 2 + 1, 0, max)) as f32) / 32768.0;

                        (cubic_interp(s1, s2, s4, s3, pos.fract()) * 32768.0) as i16
                    };
                    tmp_data.push(sr);

                    pos += phase;
                }
            } else {
                let cc = channels as usize;
                let max_samp = (num_samples - 1) * cc;

                loop {
                    if pos >= num_samples as f32 {
                        data = tmp_data;
                        break;
                    }

                    for c in 0..channels {
                        let s = unsafe {
                            let upos = pos as usize;
                            let max = max_samp + c as usize;
                            let s1 = (*data.get_unchecked(upos * cc + c as usize) as f32) / 32768.0;
                            let s2 = (*data.get_unchecked(clamp((upos + 1) * cc + c as usize, c as usize, max)) as f32)
                                / 32768.0;
                            let s3 = (*data.get_unchecked(clamp((upos + 2) * cc + c as usize, c as usize, max)) as f32) / 32768.0;
                            let s4 =
                                (*data.get_unchecked(clamp(upos.saturating_sub(1) * cc + c as usize, c as usize, max))
                                    as f32)
                                    / 32768.0;

                            (cubic_interp(s1, s2, s4, s3, pos.fract()) * 32768.0) as i16
                        };
                        tmp_data.push(s);
                    }

                    pos += phase;
                }
            }
        }

        if channels == 1 {
            let mut tmp_data = Vec::with_capacity(data.len() * 2);

            for s in data.iter().copied() {
                tmp_data.push(s);
                tmp_data.push(s);
            }

            data = tmp_data;
        }

        data
    }

    pub fn render_to(&mut self, buf: &mut [u16]) -> usize {
        while self.buffer.len() < buf.len() {
            self.decode();
        }

        self.buffer.drain(0..buf.len()).map(|n| n as u16 ^ 0x8000).zip(buf.iter_mut()).for_each(|(n, tgt)| *tgt = n);

        buf.len()
    }
}
