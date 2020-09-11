use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use bitflags::_core::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::engine_constants::EngineConstants;
use crate::ggez::{Context, filesystem, GameResult};
use crate::ggez::GameError::{AudioError, InvalidValue, ResourceLoadError};
use crate::sound::organya::Song;
use crate::sound::playback::{PlaybackEngine, SavedPlaybackState};
use crate::sound::wave_bank::SoundBank;
use crate::str;
use cpal::Sample;

pub mod pixtone;
mod wave_bank;
mod organya;
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
}

enum PlaybackMessage {
    Stop,
    PlaySong(Box<Song>),
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
    let mut engine = PlaybackEngine::new(Song::empty(), &bank);

    log::info!("Audio format: {} {}", sample_rate, channels);
    engine.set_sample_rate(sample_rate as usize);
    engine.loops = usize::MAX;

    let mut buf = vec![0x8080; 441];
    let mut index = 0;
    let mut frames = engine.render_to(&mut buf);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            match rx.try_recv() {
                Ok(PlaybackMessage::PlaySong(song)) => {
                    engine.start_song(*song, &bank);

                    for i in &mut buf[0..frames] { *i = 0x8080 };
                    frames = engine.render_to(&mut buf);
                    index = 0;

                    state = PlaybackState::Playing;
                }
                Ok(PlaybackMessage::Stop) => {
                    state = PlaybackState::Stopped;
                }
                Ok(PlaybackMessage::SetSpeed(speed)) => {
                    assert!(speed > 0.0);
                    engine.set_sample_rate((sample_rate / speed) as usize);
                }
                Ok(PlaybackMessage::SaveState) => {
                    saved_state = Some(engine.get_state());
                }
                Ok(PlaybackMessage::RestoreState) => {
                    if saved_state.is_some() {
                        engine.set_state(saved_state.clone().unwrap(), &bank);
                        saved_state = None;

                        if state == PlaybackState::Stopped {
                            engine.set_position(0);
                        }

                        for i in &mut buf[0..frames] { *i = 0x8080 };
                        frames = engine.render_to(&mut buf);
                        index = 0;

                        state = PlaybackState::Playing;
                    }
                }
                _ => {}
            }

            for frame in data.chunks_mut(channels) {
                let sample: u16 = {
                    if state == PlaybackState::Stopped {
                        0x8000
                    } else if index < frames {
                        let sample = buf[index];
                        index += 1;
                        if index & 1 == 0 { (sample & 0xff) << 8 } else { sample & 0xff00 }
                    } else {
                        for i in &mut buf[0..frames] { *i = 0x8080 };
                        frames = engine.render_to(&mut buf);
                        index = 0;
                        let sample = buf[0];
                        (sample & 0xff) << 8
                    }
                };

                let value: T = Sample::from::<u16>(&sample);
                for sample in frame.iter_mut() {
                    *sample = value;
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
