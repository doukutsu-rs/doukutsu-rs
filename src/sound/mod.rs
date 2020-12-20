use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ggez::{Context, filesystem, GameResult};
use ggez::GameError::{AudioError, InvalidValue, ResourceLoadError};
use num_traits::clamp;

use crate::engine_constants::EngineConstants;
use crate::sound::organya::Song;
use crate::sound::pixtone::PixTonePlayback;
use crate::sound::playback::{PlaybackEngine, SavedPlaybackState};
use crate::sound::wave_bank::SoundBank;
use crate::str;

mod wave_bank;
mod organya;
mod pixtone;
mod pixtone_sfx;
mod playback;
mod stuff;
mod wav;

pub struct SoundManager {
    tx: Sender<PlaybackMessage>,
    prev_song_id: usize,
    current_song_id: usize,
}

static SONGS: [&str; 43] = [
    "xxxx",
    "wanpaku",
    "anzen",
    "gameover",
    "gravity",
    "weed",
    "mdown2",
    "fireeye",
    "vivi",
    "mura",
    "fanfale1",
    "ginsuke",
    "cemetery",
    "plant",
    "kodou",
    "fanfale3",
    "fanfale2",
    "dr",
    "escape",
    "jenka",
    "maze",
    "access",
    "ironh",
    "grand",
    "curly",
    "oside",
    "requiem",
    "wanpak2",
    "quiet",
    "lastcave",
    "balcony",
    "lastbtl",
    "lastbt3",
    "ending",
    "zonbie",
    "bdown",
    "hell",
    "jenka2",
    "marine",
    "ballos",
    "toroko",
    "white",
    "kaze"
];

impl SoundManager {
    pub fn new(ctx: &mut Context) -> GameResult<SoundManager> {
        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();

        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| AudioError(str!("Error initializing audio device.")))?;
        let config = device.default_output_config()?;

        let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/pixtone.pcm")?)?;

        std::thread::spawn(move || {
            if let Err(err) = match config.sample_format() {
                cpal::SampleFormat::F32 => run::<f32>(rx, bnk, &device, &config.into()),
                cpal::SampleFormat::I16 => run::<i16>(rx, bnk, &device, &config.into()),
                cpal::SampleFormat::U16 => run::<u16>(rx, bnk, &device, &config.into()),
            } {
                log::error!("Something went wrong in audio thread: {}", err);
            }
        });

        Ok(SoundManager {
            tx: tx.clone(),
            prev_song_id: 0,
            current_song_id: 0,
        })
    }

    pub fn play_sfx(&mut self, id: u8) {
        self.tx.send(PlaybackMessage::PlaySample(id));
    }

    pub fn play_song(&mut self, song_id: usize, constants: &EngineConstants, ctx: &mut Context) -> GameResult {
        if self.current_song_id == song_id {
            return Ok(());
        }

        if song_id == 0 {
            log::info!("Stopping BGM");

            self.prev_song_id = self.current_song_id;
            self.current_song_id = 0;

            self.tx.send(PlaybackMessage::SaveState)?;
            self.tx.send(PlaybackMessage::Stop)?;
        } else if let Some(song_name) = SONGS.get(song_id) {
            let path = constants.organya_paths
                .iter()
                .map(|prefix| [prefix, &song_name.to_lowercase(), ".org"].join(""))
                .find(|path| filesystem::exists(ctx, path))
                .ok_or_else(|| ResourceLoadError(format!("BGM {:?} does not exist.", song_name)))?;

            let org = organya::Song::load_from(filesystem::open(ctx, path)?)?;
            log::info!("Playing BGM: {}", song_name);

            self.prev_song_id = self.current_song_id;
            self.current_song_id = song_id;
            self.tx.send(PlaybackMessage::SaveState)?;
            self.tx.send(PlaybackMessage::PlaySong(Box::new(org)))?;
        }
        Ok(())
    }

    pub fn save_state(&mut self) -> GameResult {
        self.tx.send(PlaybackMessage::SaveState)?;
        self.prev_song_id = self.current_song_id;

        Ok(())
    }

    pub fn restore_state(&mut self) -> GameResult {
        self.tx.send(PlaybackMessage::RestoreState)?;
        self.current_song_id = self.prev_song_id;

        Ok(())
    }

    pub fn set_speed(&mut self, speed: f32) -> GameResult {
        if speed <= 0.0 {
            return Err(InvalidValue(str!("Speed must be bigger than 0.0!")));
        }
        self.tx.send(PlaybackMessage::SetSpeed(speed))?;

        Ok(())
    }

    pub fn current_song(&self) -> usize {
        self.current_song_id
    }
}

enum PlaybackMessage {
    Stop,
    PlaySong(Box<Song>),
    PlaySample(u8),
    SetSpeed(f32),
    SaveState,
    RestoreState,
}

#[derive(PartialEq, Eq)]
enum PlaybackState {
    Stopped,
    Playing,
}

fn run<T>(rx: Receiver<PlaybackMessage>, bank: SoundBank,
          device: &cpal::Device, config: &cpal::StreamConfig) -> GameResult where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let mut state = PlaybackState::Stopped;
    let mut saved_state: Option<SavedPlaybackState> = None;
    let mut speed = 1.0;
    let mut org_engine = PlaybackEngine::new(Song::empty(), &bank);
    let mut pixtone = PixTonePlayback::new();
    pixtone.create_samples();

    log::info!("Audio format: {} {}", sample_rate, channels);
    org_engine.set_sample_rate(sample_rate as usize);
    org_engine.loops = usize::MAX;

    let buf_size = sample_rate as usize * 30 / 1000;
    let mut bgm_buf = vec![0x8080; buf_size];
    let mut pxt_buf = vec![0x8000; buf_size];
    let mut bgm_index = 0;
    let mut pxt_index = 0;
    let mut frames = org_engine.render_to(&mut bgm_buf);
    pixtone.mix(&mut pxt_buf, sample_rate);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            loop {
                match rx.try_recv() {
                    Ok(PlaybackMessage::PlaySong(song)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = None;
                        }

                        org_engine.start_song(*song, &bank);

                        for i in &mut bgm_buf[0..frames] { *i = 0x8080 };
                        frames = org_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::Playing;
                    }
                    Ok(PlaybackMessage::PlaySample(id)) => {
                        pixtone.play_sfx(id);
                    }
                    Ok(PlaybackMessage::Stop) => {
                        if state == PlaybackState::Stopped {
                            saved_state = None;
                        }

                        state = PlaybackState::Stopped;
                    }
                    Ok(PlaybackMessage::SetSpeed(new_speed)) => {
                        assert!(new_speed > 0.0);
                        speed = new_speed;
                        org_engine.set_sample_rate((sample_rate / new_speed) as usize);
                    }
                    Ok(PlaybackMessage::SaveState) => {
                        saved_state = Some(org_engine.get_state());
                    }
                    Ok(PlaybackMessage::RestoreState) => {
                        if saved_state.is_some() {
                            org_engine.set_state(saved_state.clone().unwrap(), &bank);
                            saved_state = None;

                            if state == PlaybackState::Stopped {
                                org_engine.set_position(0);
                            }

                            for i in &mut bgm_buf[0..frames] { *i = 0x8080 };
                            frames = org_engine.render_to(&mut bgm_buf);
                            bgm_index = 0;

                            state = PlaybackState::Playing;
                        }
                    }
                    Err(_) => { break; }
                }
            }

            for frame in data.chunks_mut(channels) {
                let (org_sample_l, org_sample_r): (u16, u16) = {
                    if state == PlaybackState::Stopped {
                        (0x8000, 0x8000)
                    } else if bgm_index < frames {
                        let sample = bgm_buf[bgm_index];
                        bgm_index += 1;
                        ((sample & 0xff) << 8, sample & 0xff00)
                    } else {
                        for i in &mut bgm_buf[0..frames] { *i = 0x8080 };
                        frames = org_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;
                        let sample = bgm_buf[0];
                        ((sample & 0xff) << 8, sample & 0xff00)
                    }
                };
                let pxt_sample: u16 = pxt_buf[pxt_index];

                if pxt_index < (pxt_buf.len() - 1) {
                    pxt_index += 1;
                } else {
                    pxt_index = 0;
                    for i in pxt_buf.iter_mut() { *i = 0x8000 };
                    pixtone.mix(&mut pxt_buf, sample_rate / speed);
                }

                if frame.len() >= 2 {
                    let sample_l = clamp(
                        (((org_sample_l ^ 0x8000) as i16) as isize)
                            + (((pxt_sample ^ 0x8000) as i16) as isize)
                        , -0x7fff, 0x7fff) as u16 ^ 0x8000;
                    let sample_r = clamp(
                        (((org_sample_r ^ 0x8000) as i16) as isize)
                            + (((pxt_sample ^ 0x8000) as i16) as isize)
                        , -0x7fff, 0x7fff) as u16 ^ 0x8000;

                    frame[0] = Sample::from::<u16>(&sample_l);
                    frame[1] = Sample::from::<u16>(&sample_r);
                } else {
                    let sample = clamp(
                        (((org_sample_l ^ 0x8000) as i16) as isize)
                            + (((pxt_sample ^ 0x8000) as i16) as isize)
                        , -0x7fff, 0x7fff) as u16 ^ 0x8000;

                    frame[0] = Sample::from::<u16>(&sample);
                }
            }
        },
        err_fn,
    )?;
    stream.play()?;

    loop {
        std::thread::sleep(Duration::from_millis(10));
    }
}
