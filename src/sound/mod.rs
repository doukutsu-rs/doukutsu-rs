use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use bitflags::_core::time::Duration;
use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::engine_constants::EngineConstants;
use crate::ggez::{Context, filesystem, GameResult};
use crate::ggez::GameError::{AudioError, InvalidValue, ResourceLoadError};
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
    let mut engine = PlaybackEngine::new(Song::empty(), &bank);
    let mut pixtone = PixTonePlayback::new();
    pixtone.create_samples();


    log::info!("Audio format: {} {}", sample_rate, channels);
    engine.set_sample_rate(sample_rate as usize);
    engine.loops = usize::MAX;

    let mut org_buf = vec![0x8080; 441];
    let mut pxt_buf = vec![0x8000; 441];
    let mut org_index = 0;
    let mut pxt_index = 0;
    let mut frames = engine.render_to(&mut org_buf);
    pixtone.mix(&mut pxt_buf, sample_rate);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            loop {
                match rx.try_recv() {
                    Ok(PlaybackMessage::PlaySong(song)) => {
                        engine.start_song(*song, &bank);

                        for i in &mut org_buf[0..frames] { *i = 0x8080 };
                        frames = engine.render_to(&mut org_buf);
                        org_index = 0;

                        state = PlaybackState::Playing;
                    }
                    Ok(PlaybackMessage::PlaySample(id)) => {
                        pixtone.play_sfx(id);
                    }
                    Ok(PlaybackMessage::Stop) => {
                        state = PlaybackState::Stopped;
                    }
                    Ok(PlaybackMessage::SetSpeed(new_speed)) => {
                        assert!(new_speed > 0.0);
                        speed = new_speed;
                        engine.set_sample_rate((sample_rate / new_speed) as usize);
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

                            for i in &mut org_buf[0..frames] { *i = 0x8080 };
                            frames = engine.render_to(&mut org_buf);
                            org_index = 0;

                            state = PlaybackState::Playing;
                        }
                    }
                    Err(_) => { break; }
                }
            }

            for frame in data.chunks_mut(channels) {
                let org_sample: u16 = {
                    if state == PlaybackState::Stopped {
                        0x8000
                    } else if org_index < frames {
                        let sample = org_buf[org_index];
                        org_index += 1;
                        if org_index & 1 == 0 { (sample & 0xff) << 8 } else { sample & 0xff00 }
                    } else {
                        for i in &mut org_buf[0..frames] { *i = 0x8080 };
                        frames = engine.render_to(&mut org_buf);
                        org_index = 0;
                        let sample = org_buf[0];
                        (sample & 0xff) << 8
                    }
                };
                let pxt_sample: u16 = pxt_buf[pxt_index] ^ 0x8000;

                if pxt_index < (pxt_buf.len() - 1) {
                    pxt_index += 1;
                } else {
                    pxt_index = 0;
                    for i in pxt_buf.iter_mut() { *i = 0x8000 };
                    pixtone.mix(&mut pxt_buf, sample_rate / speed);
                }

                let sample = org_sample.wrapping_add(pxt_sample);

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
        std::thread::sleep(Duration::from_millis(4));
    }
}
