use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Instant;
use num_traits::clamp;
use crate::framework::filesystem::File;
use organya::org_playback::OrgPlaybackEngine;
use crate::sound::ogg_playback::OggPlaybackEngine;
use organya::pixtone::PixTonePlayback;
use organya::wave_bank::SoundBank;
use crate::sound::{PlaybackMessage, PlaybackState, PlaybackStateType, InterpolationMode, FADEOUT_DURATION};

pub struct DoukutsuMixer {
    rx: Receiver<PlaybackMessage>,
    bank: SoundBank,
    sample_rate: f32,
    channels: u16,
    state: PlaybackState,
    saved_state: PlaybackStateType,
    speed: f32,
    org_engine: Box<OrgPlaybackEngine>,
    ogg_engine: Box<OggPlaybackEngine>,
    pixtone: Box<PixTonePlayback>,
    
    bgm_buf: Vec<u16>,
    pxt_buf: Vec<u16>,
    bgm_index: usize,
    pxt_index: usize,
    samples: usize,
    bgm_vol: f32,
    bgm_vol_saved: f32,
    sfx_vol: f32,
    bgm_fadeout: bool,
    last_fadeout: Instant,
    initialized: bool,
}

impl DoukutsuMixer {
    pub fn new(rx: Receiver<PlaybackMessage>, bank: SoundBank, sample_rate: f32, channels: u16) -> Self {
        let mut org_engine = Box::new(OrgPlaybackEngine::new());
        let mut ogg_engine = Box::new(OggPlaybackEngine::new());
        let pixtone = Box::new(PixTonePlayback::new());

        org_engine.set_sample_rate(sample_rate as usize);
        org_engine.loops = usize::MAX;
        ogg_engine.set_sample_rate(sample_rate as usize);

        let buf_size = sample_rate as usize * 10 / 1000;
        let pxt_buf = vec![0x8000; buf_size];

        Self {
            rx,
            bank,
            sample_rate,
            channels,
            state: PlaybackState::Stopped,
            saved_state: PlaybackStateType::None,
            speed: 1.0,
            org_engine,
            ogg_engine,
            pixtone,
            bgm_buf: vec![0x8000; buf_size * 2],
            pxt_buf,
            bgm_index: 0,
            pxt_index: 0,
            samples: 0,
            bgm_vol: 1.0,
            bgm_vol_saved: 1.0,
            sfx_vol: 1.0,
            bgm_fadeout: false,
            last_fadeout: Instant::now(),
            initialized: false,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.org_engine.set_sample_rate((sample_rate / self.speed) as usize);
        self.ogg_engine.set_sample_rate((sample_rate / self.speed) as usize);
        
        let buf_size = sample_rate as usize * 10 / 1000;
        self.bgm_buf.resize(buf_size * 2, 0x8000);
        self.pxt_buf.resize(buf_size, 0x8000);
        self.bgm_index = 0;
        self.pxt_index = 0;
        self.samples = 0;
    }

    pub fn set_channels(&mut self, channels: u16) {
        self.channels = channels;
    }

    pub fn mix(&mut self, data: &mut [f32]) {
        if !self.initialized {
            self.pixtone.create_samples();
            self.pixtone.mix(&mut self.pxt_buf, self.sample_rate);
            self.initialized = true;
        }

        loop {
            match self.rx.try_recv() {
                Ok(msg) => self.handle_message(msg),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        if self.bgm_fadeout && self.bgm_vol > 0.0 {
            let elapsed = self.last_fadeout.elapsed().as_secs_f32();
            self.last_fadeout = Instant::now();
            let step = self.bgm_vol_saved / FADEOUT_DURATION * elapsed;
            self.bgm_vol = (self.bgm_vol - step).max(0.0);
        }

        let channels = self.channels as usize;
        for frame in data.chunks_mut(channels) {
            let (bgm_sample_l, bgm_sample_r): (u16, u16) = {
                if self.state == PlaybackState::Stopped {
                    (0x8000, 0x8000)
                } else if self.bgm_index < self.samples {
                    let samples = (self.bgm_buf[self.bgm_index], self.bgm_buf[self.bgm_index + 1]);
                    self.bgm_index += 2;
                    samples
                } else {
                    self.bgm_buf.fill(0x8000);
                    match self.state {
                        PlaybackState::PlayingOrg => {
                            self.samples = self.org_engine.render_to(&mut self.bgm_buf);
                        }
                        PlaybackState::PlayingOgg => {
                            self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                        }
                        _ => unreachable!(),
                    }
                    self.bgm_index = 2;
                    (self.bgm_buf[0], self.bgm_buf[1])
                }
            };

            let pxt_sample: u16 = self.pxt_buf[self.pxt_index];

            if self.pxt_index < (self.pxt_buf.len() - 1) {
                self.pxt_index += 1;
            } else {
                self.pxt_index = 0;
                self.pxt_buf.fill(0x8000);
                self.pixtone.mix(&mut self.pxt_buf, self.sample_rate / self.speed);
            }

            if frame.len() >= 2 {
                let sample_l = clamp(
                    (((bgm_sample_l ^ 0x8000) as i16) as f32 * self.bgm_vol)
                        + (((pxt_sample ^ 0x8000) as i16) as f32 * self.sfx_vol),
                    -32767.0,
                    32767.0,
                ) / 32768.0;
                let sample_r = clamp(
                    (((bgm_sample_r ^ 0x8000) as i16) as f32 * self.bgm_vol)
                        + (((pxt_sample ^ 0x8000) as i16) as f32 * self.sfx_vol),
                    -32767.0,
                    32767.0,
                ) / 32768.0;

                frame[0] = sample_l;
                frame[1] = sample_r;
                for i in 2..frame.len() {
                    frame[i] = 0.0;
                }
            } else if !frame.is_empty() {
                let sample = clamp(
                    ((((bgm_sample_l ^ 0x8000) as i16) + ((bgm_sample_r ^ 0x8000) as i16)) as f32 * self.bgm_vol / 2.0)
                        + (((pxt_sample ^ 0x8000) as i16) as f32 * self.sfx_vol),
                    -32767.0,
                    32767.0,
                ) / 32768.0;

                frame[0] = sample;
            }
        }
    }

    fn handle_message(&mut self, msg: PlaybackMessage) {
        match msg {
            PlaybackMessage::PlayOrganyaFile(file) => {
                if self.state == PlaybackState::Stopped {
                    self.saved_state = PlaybackStateType::None;
                }
                if self.bgm_fadeout {
                    self.bgm_fadeout = false;
                    self.bgm_vol = self.bgm_vol_saved;
                }

                match organya::organya::Song::load_from(file) {
                    Ok(song) => {
                        self.org_engine.start_song(song, &self.bank);
                        self.bgm_buf.fill(0x8000);
                        self.samples = self.org_engine.render_to(&mut self.bgm_buf);
                        self.bgm_index = 0;
                        self.state = PlaybackState::PlayingOrg;
                    }
                    Err(e) => {
                        log::error!("Error loading Organya song in mixer: {}", e);
                    }
                }
            }
            PlaybackMessage::PlayOggSongSinglePartFile(file) => {
                if self.state == PlaybackState::Stopped {
                    self.saved_state = PlaybackStateType::None;
                }
                if self.bgm_fadeout {
                    self.bgm_fadeout = false;
                    self.bgm_vol = self.bgm_vol_saved;
                }

                match lewton::inside_ogg::OggStreamReader::new(file) {
                    Ok(song) => {
                        self.ogg_engine.start_single(Box::new(song));
                        self.bgm_buf.fill(0x8000);
                        self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                        self.bgm_index = 0;
                        self.state = PlaybackState::PlayingOgg;
                    }
                    Err(e) => {
                        log::error!("Error loading Ogg song in mixer: {}", e);
                    }
                }
            }
            PlaybackMessage::PlayOggSongMultiPartFile(file_intro, file_loop) => {
                if self.state == PlaybackState::Stopped {
                    self.saved_state = PlaybackStateType::None;
                }
                if self.bgm_fadeout {
                    self.bgm_fadeout = false;
                    self.bgm_vol = self.bgm_vol_saved;
                }

                match (lewton::inside_ogg::OggStreamReader::new(file_intro), lewton::inside_ogg::OggStreamReader::new(file_loop)) {
                    (Ok(song_intro), Ok(song_loop)) => {
                        self.ogg_engine.start_multi(Box::new(song_intro), Box::new(song_loop));
                        self.bgm_buf.fill(0x8000);
                        self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                        self.bgm_index = 0;
                        self.state = PlaybackState::PlayingOgg;
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        log::error!("Error loading multi-part Ogg song in mixer: {}", e);
                    }
                }
            }
            PlaybackMessage::PlaySample(id) => {
                self.pixtone.play_sfx(id);
            }
            PlaybackMessage::LoopSample(id) => {
                self.pixtone.loop_sfx(id);
            }
            PlaybackMessage::LoopSampleFreq(id, freq) => {
                self.pixtone.loop_sfx_freq(id, freq);
            }
            PlaybackMessage::StopSample(id) => {
                self.pixtone.stop_sfx(id);
            }
            PlaybackMessage::Stop => {
                if self.state == PlaybackState::Stopped {
                    self.saved_state = PlaybackStateType::None;
                }
                self.state = PlaybackState::Stopped;
            }
            PlaybackMessage::SetSpeed(new_speed) => {
                self.speed = new_speed;
                self.org_engine.set_sample_rate((self.sample_rate / new_speed) as usize);
                self.ogg_engine.set_sample_rate((self.sample_rate / new_speed) as usize);
            }
            PlaybackMessage::SetSongVolume(new_volume) => {
                if self.bgm_fadeout {
                    self.bgm_vol_saved = new_volume;
                } else {
                    self.bgm_vol = new_volume;
                }
            }
            PlaybackMessage::SetSampleVolume(new_volume) => {
                self.sfx_vol = new_volume;
            }
            PlaybackMessage::FadeoutSong => {
                self.bgm_fadeout = true;
                self.bgm_vol_saved = self.bgm_vol;
                self.last_fadeout = Instant::now();
            }
            PlaybackMessage::SaveState => {
                self.saved_state = match self.state {
                    PlaybackState::Stopped => PlaybackStateType::None,
                    PlaybackState::PlayingOrg => PlaybackStateType::Organya(self.org_engine.get_state()),
                    PlaybackState::PlayingOgg => PlaybackStateType::Ogg(self.ogg_engine.get_state()),
                };
            }
            PlaybackMessage::RestoreState => {
                let saved_state_loc = std::mem::take(&mut self.saved_state);
                match saved_state_loc {
                    PlaybackStateType::None => {
                        self.state = PlaybackState::Stopped;
                    }
                    PlaybackStateType::Organya(playback_state) => {
                        self.org_engine.set_state(playback_state, &self.bank);
                        if self.state == PlaybackState::Stopped {
                            self.org_engine.rewind();
                        }
                        self.bgm_buf.fill(0x8000);
                        self.samples = self.org_engine.render_to(&mut self.bgm_buf);
                        self.bgm_index = 0;
                        if self.bgm_fadeout {
                            self.bgm_fadeout = false;
                            self.bgm_vol = self.bgm_vol_saved;
                        }
                        self.state = PlaybackState::PlayingOrg;
                    }
                    PlaybackStateType::Ogg(playback_state) => {
                        self.ogg_engine.set_state(playback_state);
                        if self.state == PlaybackState::Stopped {
                            self.ogg_engine.rewind();
                        }
                        self.bgm_buf.fill(0x8000);
                        self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                        self.bgm_index = 0;
                        if self.bgm_fadeout {
                            self.bgm_fadeout = false;
                            self.bgm_vol = self.bgm_vol_saved;
                        }
                        self.state = PlaybackState::PlayingOgg;
                    }
                }
            }
            PlaybackMessage::SetSampleParams(id, params) => {
                self.pixtone.set_sample_parameters(id, params);
            }
            PlaybackMessage::SetOrgInterpolation(interpolation) => {
                self.org_engine.interpolation = interpolation;
            }
            PlaybackMessage::SetSampleData(id, data) => {
                self.pixtone.set_sample_data(id, data);
            }
        }
    }
}
