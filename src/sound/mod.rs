use std::io;
use std::io::{BufRead, BufReader, Lines};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::{Condvar, mpsc};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
#[cfg(feature = "ogg-playback")]
use lewton::inside_ogg::OggStreamReader;
use num_traits::clamp;

use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameError::{AudioError, InvalidValue};
use crate::framework::error::{GameError, GameResult};
use crate::framework::filesystem;
use crate::framework::filesystem::File;
use crate::game::settings::Settings;
#[cfg(feature = "ogg-playback")]
use crate::sound::ogg_playback::{OggPlaybackEngine, SavedOggPlaybackState};
use crate::sound::org_playback::{OrgPlaybackEngine, SavedOrganyaPlaybackState};
use crate::sound::organya::Song;
use crate::sound::pixtone::{PixToneParameters, PixTonePlayback};
use crate::sound::wave_bank::SoundBank;

mod fir;
#[cfg(feature = "ogg-playback")]
mod ogg_playback;
mod org_playback;
mod organya;
pub mod pixtone;
mod pixtone_sfx;
mod stuff;
mod wav;
mod wave_bank;

pub struct SoundManager {
    soundbank: Option<SoundBank>,
    tx: Sender<PlaybackMessage>,
    prev_song_id: usize,
    current_song_id: usize,
    no_audio: bool,
    load_failed: bool,

    ctx: Arc<Mutex<AudioContext>>,
    watchdog_tx: std::sync::mpsc::Sender<WatchDogMessage>,
    watchdog_condvar: Arc<Condvar>,
    watchdog: Option<std::thread::JoinHandle<GameResult<()>>>,
}

enum SongFormat {
    Organya,
    #[cfg(feature = "ogg-playback")]
    OggSinglePart,
    #[cfg(feature = "ogg-playback")]
    OggMultiPart,
}

#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InterpolationMode {
    Nearest,
    Linear,
    Cosine,
    Cubic,
    Polyphase,
}

impl SoundManager {
    pub fn new(ctx: &mut Context) -> GameResult<SoundManager> {
        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = bounded(4096);
        let (watchdog_tx, watchdog_rx) = mpsc::channel::<WatchDogMessage>();
        let audio_ctx = Arc::new(Mutex::new(AudioContext::new()));

        if ctx.headless {
            log::info!("Running in headless mode, skipping initialization.");

            return Ok(SoundManager {
                soundbank: None,
                tx: tx.clone(),
                prev_song_id: 0,
                current_song_id: 0,
                no_audio: true,
                load_failed: false,

                ctx: audio_ctx,
                watchdog_tx: watchdog_tx,
                watchdog_condvar: Arc::new(Condvar::new()),
                watchdog: None,
            });
        }

        let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/organya-wavetable-doukutsu.bin")?)?;
        Ok(SoundManager::bootstrap(&bnk, audio_ctx, tx, rx, watchdog_tx, watchdog_rx)?)
    }

    fn bootstrap(
        soundbank: &SoundBank,
        audio_ctx: Arc<Mutex<AudioContext>>,
        tx: Sender<PlaybackMessage>,
        rx: Receiver<PlaybackMessage>,
        watchdog_tx: std::sync::mpsc::Sender<WatchDogMessage>,
        watchdog_rx: std::sync::mpsc::Receiver<WatchDogMessage>,
    ) -> GameResult<SoundManager> {
        let mut sound_manager = SoundManager {
            soundbank: Some(soundbank.to_owned()),
            tx: tx.clone(),
            prev_song_id: 0,
            current_song_id: 0,
            no_audio: false,
            load_failed: false,

            ctx: audio_ctx,
            watchdog_tx: watchdog_tx,
            watchdog_condvar: Arc::new(Condvar::new()),
            watchdog: None,
        };

        let res =
            audio_watchdog_bootstrap(sound_manager.ctx.to_owned(), soundbank.to_owned(), tx.clone(), rx, watchdog_rx, sound_manager.watchdog_condvar.clone());
        if let Err(res) = &res {
            log::error!("Error initializing audio: {}", res);
            sound_manager.load_failed = true;
        }

        Ok(sound_manager)
    }

    pub fn reload(&mut self) -> GameResult<()> {
        if self.no_audio {
            log::info!("Skipping sound manager reload because audio is not enabled.");
            return Ok(());
        }

        log::info!("Reloading sound manager.");

        if self.watchdog_tx.send(WatchDogMessage::Reload).is_err() {
            let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = bounded(4096);
            let (watchdog_tx, watchdog_rx) = mpsc::channel::<WatchDogMessage>();

            let soundbank = self.soundbank.take().unwrap();
            *self = SoundManager::bootstrap(&soundbank, self.ctx.clone(), tx, rx, watchdog_tx, watchdog_rx)?;
        }
        self.watchdog_condvar.notify_one();

        Ok(())
    }

    fn send(&mut self, message: PlaybackMessage) -> GameResult<()> {
        if self.no_audio {
            return Ok(());
        }

        if self.tx.send(message).is_err() {
            if !self.load_failed {
                log::error!("Error sending message to audio thread. Press Ctrl + F3 to reload sound manager.");
                self.reload()?;
            }
        }
        //self.watchdog_condvar.notify_one();

        Ok(())
    }

    pub fn pause(&mut self) {
        if self.watchdog_tx.send(WatchDogMessage::Pause).is_err() {
            // Watchdog is possibly destroyed, try to create a new one
            let _ = self.reload();
        }
    }

    pub fn resume(&mut self) {
        if self.watchdog_tx.send(WatchDogMessage::Play).is_err() {
            // Watchdog is possibly destroyed, try to create a new one
            let _ = self.reload();
        }
    }

    pub fn play_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }

        self.send(PlaybackMessage::PlaySample(id)).unwrap();
    }

    pub fn loop_sfx(&self, id: u8) {
        if self.no_audio {
            return;
        }

        self.tx.send(PlaybackMessage::LoopSample(id)).unwrap();
    }

    pub fn loop_sfx_freq(&mut self, id: u8, freq: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::LoopSampleFreq(id, freq)).unwrap();
    }

    pub fn stop_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::StopSample(id)).unwrap();
    }

    pub fn set_org_interpolation(&mut self, interpolation: InterpolationMode) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetOrgInterpolation(interpolation)).unwrap();
    }

    pub fn set_song_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSongVolume(volume.powf(3.0))).unwrap();
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleVolume(volume.powf(3.0))).unwrap();
    }

    pub fn set_sfx_samples(&mut self, id: u8, data: Vec<i16>) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleData(id, data)).unwrap();
    }

    pub fn reload_songs(&mut self, constants: &EngineConstants, settings: &Settings, ctx: &mut Context) -> GameResult {
        let prev_song = self.prev_song_id;
        let current_song = self.current_song_id;

        self.play_song(0, constants, settings, ctx, false)?;
        self.play_song(prev_song, constants, settings, ctx, false)?;
        self.save_state()?;
        self.play_song(current_song, constants, settings, ctx, false)?;

        Ok(())
    }

    pub fn play_song(
        &mut self,
        song_id: usize,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult {
        if self.current_song_id == song_id || self.no_audio {
            return Ok(());
        }

        if song_id == 0 {
            log::info!("Stopping BGM");

            self.prev_song_id = self.current_song_id;
            self.current_song_id = 0;

            self.send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation)).unwrap();
            self.send(PlaybackMessage::SaveState).unwrap();

            if fadeout {
                self.send(PlaybackMessage::FadeoutSong).unwrap();
            } else {
                self.send(PlaybackMessage::Stop).unwrap();
            }
        } else if let Some(song_name) = constants.music_table.get(song_id) {
            let mut paths = constants.organya_paths.clone();

            paths.insert(0, "/Soundtracks/".to_owned() + &settings.soundtrack + "/");

            if let Some(soundtrack) = constants.soundtracks.iter().find(|s| s.available && s.id == settings.soundtrack)
            {
                paths.insert(0, soundtrack.path.clone());
            }

            let songs_paths = paths.iter().map(|prefix| {
                [
                    #[cfg(feature = "ogg-playback")]
                    (
                        SongFormat::OggMultiPart,
                        vec![format!("{}{}_intro.ogg", prefix, song_name), format!("{}{}_loop.ogg", prefix, song_name)],
                    ),
                    #[cfg(feature = "ogg-playback")]
                    (SongFormat::OggSinglePart, vec![format!("{}{}.ogg", prefix, song_name)]),
                    (SongFormat::Organya, vec![format!("{}{}.org", prefix, song_name)]),
                ]
            });

            for songs in songs_paths {
                for (format, paths) in
                    songs.iter().filter(|(_, paths)| paths.iter().all(|path| filesystem::exists(ctx, path)))
                {
                    match format {
                        SongFormat::Organya => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(organya::Song::load_from) {
                                Ok(Ok(org)) => {
                                    log::info!("Playing Organya BGM: {} {}", song_id, path);

                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    let _ = self
                                        .send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation))
                                        .unwrap();
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOrganyaSong(Box::new(org))).unwrap();

                                    return Ok(());
                                }
                                Ok(Err(err)) | Err(err) => {
                                    log::warn!("Failed to load Organya BGM {}: {}", song_id, err);
                                }
                            }
                        }
                        #[cfg(feature = "ogg-playback")]
                        SongFormat::OggSinglePart => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(|f| {
                                OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                            }) {
                                Ok(Ok(song)) => {
                                    log::info!("Playing single part Ogg BGM: {} {}", song_id, path);

                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOggSongSinglePart(Box::new(song))).unwrap();

                                    return Ok(());
                                }
                                Ok(Err(err)) | Err(err) => {
                                    log::warn!("Failed to load single part Ogg BGM {}: {}", song_id, err);
                                }
                            }
                        }
                        #[cfg(feature = "ogg-playback")]
                        SongFormat::OggMultiPart => {
                            // we're sure that there are two elements
                            let path_intro = unsafe { paths.get_unchecked(0) };
                            let path_loop = unsafe { paths.get_unchecked(1) };

                            match (
                                filesystem::open(ctx, path_intro).map(|f| {
                                    OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                                }),
                                filesystem::open(ctx, path_loop).map(|f| {
                                    OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                                }),
                            ) {
                                (Ok(Ok(song_intro)), Ok(Ok(song_loop))) => {
                                    log::info!(
                                        "Playing multi part Ogg BGM: {} {} + {}",
                                        song_id,
                                        path_intro,
                                        path_loop
                                    );

                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOggSongMultiPart(
                                        Box::new(song_intro),
                                        Box::new(song_loop),
                                    ))
                                    .unwrap();

                                    return Ok(());
                                }
                                (Ok(Err(err)), _) | (Err(err), _) | (_, Ok(Err(err))) | (_, Err(err)) => {
                                    log::warn!("Failed to load multi part Ogg BGM {}: {}", song_id, err);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn save_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SaveState).unwrap();
        self.prev_song_id = self.current_song_id;

        Ok(())
    }

    pub fn restore_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::RestoreState).unwrap();
        self.current_song_id = self.prev_song_id;

        Ok(())
    }

    pub fn set_speed(&mut self, speed: f32) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        if speed <= 0.0 {
            return Err(InvalidValue("Speed must be bigger than 0.0!".to_owned()));
        }

        self.send(PlaybackMessage::SetSpeed(speed)).unwrap();

        Ok(())
    }

    pub fn current_song(&self) -> usize {
        self.current_song_id
    }

    pub fn set_sample_params_from_file<R: io::Read>(&mut self, id: u8, data: R) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        let mut reader = BufReader::new(data).lines();
        let mut params = PixToneParameters::empty();

        fn next_string<T: FromStr, R: io::Read>(reader: &mut Lines<BufReader<R>>) -> GameResult<T> {
            while let Some(Ok(str)) = reader.next() {
                let str = str.trim();
                if str.is_empty() || str.starts_with('#') {
                    continue;
                }

                let mut splits = str.split(':');

                let _ = splits.next();
                if let Some(str) = splits.next() {
                    return str.trim().parse::<T>().map_err(|_| {
                        GameError::ParseError("failed to parse the value as specified type.".to_string())
                    });
                } else {
                    break;
                }
            }

            Err(GameError::ParseError("unexpected end.".to_string()))
        }

        for channel in &mut params.channels {
            channel.enabled = next_string::<u8, R>(&mut reader)? != 0;
            channel.length = next_string::<u32, R>(&mut reader)?;

            channel.carrier.waveform_type = next_string::<u8, R>(&mut reader)?;
            channel.carrier.pitch = next_string::<f32, R>(&mut reader)?;
            channel.carrier.level = next_string::<i32, R>(&mut reader)?;
            channel.carrier.offset = next_string::<i32, R>(&mut reader)?;

            channel.frequency.waveform_type = next_string::<u8, R>(&mut reader)?;
            channel.frequency.pitch = next_string::<f32, R>(&mut reader)?;
            channel.frequency.level = next_string::<i32, R>(&mut reader)?;
            channel.frequency.offset = next_string::<i32, R>(&mut reader)?;

            channel.amplitude.waveform_type = next_string::<u8, R>(&mut reader)?;
            channel.amplitude.pitch = next_string::<f32, R>(&mut reader)?;
            channel.amplitude.level = next_string::<i32, R>(&mut reader)?;
            channel.amplitude.offset = next_string::<i32, R>(&mut reader)?;

            channel.envelope.initial = next_string::<i32, R>(&mut reader)?;
            channel.envelope.time_a = next_string::<i32, R>(&mut reader)?;
            channel.envelope.value_a = next_string::<i32, R>(&mut reader)?;
            channel.envelope.time_b = next_string::<i32, R>(&mut reader)?;
            channel.envelope.value_b = next_string::<i32, R>(&mut reader)?;
            channel.envelope.time_c = next_string::<i32, R>(&mut reader)?;
            channel.envelope.value_c = next_string::<i32, R>(&mut reader)?;
        }

        self.set_sample_params(id, params)
    }

    pub fn set_sample_params(&mut self, id: u8, params: PixToneParameters) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SetSampleParams(id, params)).unwrap();

        Ok(())
    }

    pub fn load_custom_sound_effects(&mut self, ctx: &mut Context, roots: &Vec<String>) -> GameResult {
        for path in roots.iter().rev() {
            let wavs = filesystem::read_dir(ctx, [path, "sfx/"].join(""))?
                .filter(|f| f.to_string_lossy().to_lowercase().ends_with(".wav"));

            for filename in wavs {
                if let Ok(mut file) = filesystem::open(ctx, &filename) {
                    let wav = wav::WavSample::read_from(&mut file)?;
                    let id = filename
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .parse::<u8>()
                        .unwrap_or(0);
                    if id == 0 {
                        continue;
                    }
                    let step = (wav.format.channels * 2) as usize;
                    let data = wav
                        .data
                        .chunks_exact(2)
                        .into_iter()
                        .step_by(step)
                        .map(|a| i16::from_ne_bytes([a[0], a[1]]))
                        .collect();

                    self.set_sfx_samples(id, data);
                }
            }
        }
        Ok(())
    }
}

pub(in crate::sound) enum WatchDogMessage {
    Reload,
    Play,
    Pause,
}

pub(in crate::sound) struct AudioContext {
    pub bgm_vol: f32,
    pub bgm_vol_saved: f32,
    pub bgm_fadeout: bool,
    pub sfx_vol: f32,

    pub bgm_buf: Vec<u16>,
    pub pxt_buf: Vec<u16>,
    pub bgm_index: usize,
    pub pxt_index: usize,
    pub samples: usize,

    pub state: PlaybackState,
    pub saved_state: PlaybackStateType,
    pub speed: f32,

    pub org_engine: Box<OrgPlaybackEngine>,
    #[cfg(feature = "ogg-playback")]
    pub ogg_engine: Box<OggPlaybackEngine>,
    pub pixtone: Box<PixTonePlayback>,
}

impl AudioContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            bgm_vol: 0.0,
            bgm_vol_saved: 0.0,
            bgm_fadeout: false,
            sfx_vol: 0.0,

            bgm_buf: Vec::new(),
            pxt_buf: Vec::new(),
            bgm_index: 0,
            pxt_index: 0,
            samples: 0,

            state: PlaybackState::Stopped,
            saved_state: PlaybackStateType::None,
            speed: 1.0,

            org_engine: Box::new(OrgPlaybackEngine::new()),
            #[cfg(feature = "ogg-playback")]
            ogg_engine: Box::new(OggPlaybackEngine::new()),
            pixtone: Box::new(PixTonePlayback::new()),
        };
        ctx.pixtone.create_samples();

        ctx
    }

    pub fn device_changed(&mut self, sample_rate: f32) {
        self.org_engine.set_sample_rate(sample_rate as usize);
        #[cfg(feature = "ogg-playback")]
        {
            self.org_engine.loops = usize::MAX;
            self.ogg_engine.set_sample_rate(sample_rate as usize);
        }
        self.pixtone.mix(self.pxt_buf.as_mut_slice(), sample_rate);

        let buf_size = sample_rate as usize * 10 / 1000;
        self.bgm_buf.resize(buf_size * 2, 0x8080);
        self.pxt_buf.resize(buf_size, 0x8000);

        // TODO: add handling of sample rate change for buffer contents
    }
}

pub(in crate::sound) enum PlaybackMessage {
    Stop,
    PlayOrganyaSong(Box<Song>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongSinglePart(Box<OggStreamReader<File>>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongMultiPart(Box<OggStreamReader<File>>, Box<OggStreamReader<File>>),
    PlaySample(u8),
    LoopSample(u8),
    LoopSampleFreq(u8, f32),
    StopSample(u8),
    SetSpeed(f32),
    SetSongVolume(f32),
    SetSampleVolume(f32),
    FadeoutSong,
    SaveState,
    RestoreState,
    SetSampleParams(u8, PixToneParameters),
    SetOrgInterpolation(InterpolationMode),
    SetSampleData(u8, Vec<i16>),
}

#[derive(PartialEq, Eq)]
enum PlaybackState {
    Stopped,
    PlayingOrg,
    #[cfg(feature = "ogg-playback")]
    PlayingOgg,
}

enum PlaybackStateType {
    None,
    Organya(SavedOrganyaPlaybackState),
    #[cfg(feature = "ogg-playback")]
    Ogg(SavedOggPlaybackState),
}

impl Default for PlaybackStateType {
    fn default() -> Self {
        Self::None
    }
}

fn obtain_audio_device() -> GameResult<(cpal::Device, cpal::SupportedStreamConfig)> {
    let host = cpal::default_host();

    let device =
        host.default_output_device().ok_or_else(|| AudioError("Error initializing audio device.".to_owned()))?;

    let config = device.default_output_config()?;
    Ok((device, config))
}

fn audio_watchdog_bootstrap(
    audio_ctx: Arc<Mutex<AudioContext>>,
    soundbank: SoundBank,
    tx: Sender<PlaybackMessage>,
    rx: Receiver<PlaybackMessage>,
    master_rx: std::sync::mpsc::Receiver<WatchDogMessage>,
    condvar: Arc<Condvar>
) -> GameResult<std::thread::JoinHandle<GameResult<()>>> {
    let builder = std::thread::Builder::new().name("Audio watchdog".to_owned());
    let handle = builder.spawn(move || {
        let (mut err_tx, mut err_rx) = mpsc::channel::<cpal::StreamError>();
        let mut stream: Option<cpal::Stream> = None;
        let dummy_mutex = Mutex::new(0);
        let mut dummy_lock = dummy_mutex.lock().unwrap();

        loop {
            dummy_lock = condvar.wait_timeout(dummy_lock, Duration::from_secs(3)).unwrap().0;
            if stream.is_none() {
                let device_result = obtain_audio_device();
                if device_result.is_err() {
                    log::error!("Failed to obtain new audio device. Retry in a few seconds.");
                    continue;
                }

                let (device, config) = device_result.unwrap();
                let res = match config.sample_format() {
                    cpal::SampleFormat::I8 => run::<i8>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::I16 => run::<i16>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::I32 => run::<i32>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::I64 => run::<i64>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::U8 => run::<u8>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::U16 => run::<u16>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::U32 => run::<u32>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::U64 => run::<u64>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::F32 => run::<f32>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    cpal::SampleFormat::F64 => run::<f64>(rx.clone(), err_tx.clone(), soundbank.to_owned(), device, config.into(), audio_ctx.clone(), condvar.clone()),
                    _ => Err(AudioError("Unsupported sample format.".to_owned())),
                };
                if let Err(e) = res {
                    log::error!("Failed to run audio stream due to error: {}. Try to obtain new device in a few seconds.", e.to_string());
                    continue;
                }

                stream = res.ok();
            }

            let mut no_errors = match err_rx.try_recv() {
                Ok(err) => {
                    log::error!("Error occured on audio stream: {}", err);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => false,
                _ => true,
            };
            if !no_errors && stream.is_some() {
                drop(stream.take());
                continue;
            }

            match master_rx.try_recv() {
                Ok(WatchDogMessage::Reload) => {
                    if stream.is_some() {
                        drop(stream.take());
                        continue;
                    }
                }
                Ok(WatchDogMessage::Play) => {
                    if let Some(stream_handle) = &mut stream {
                        let _ = stream_handle.play();
                    }
                }
                Ok(WatchDogMessage::Pause) => {
                    if let Some(stream_handle) = &mut stream {
                        let _ = stream_handle.pause();
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err(AudioError("Lost connection with master thread.".to_owned()));
                }
                _ => continue,
            }
        }
    });

    if let Err(e) = handle {
        return Err(AudioError(format!("Failed to spawn audio watchdog thread. {}", e.to_string())));
    }

    Ok(handle.unwrap())
}

fn run<T>(
    rx: Receiver<PlaybackMessage>,
    err_tx: std::sync::mpsc::Sender<cpal::StreamError>,
    bank: SoundBank,
    device: cpal::Device,
    config: cpal::StreamConfig,
    audio_ctx: Arc<Mutex<AudioContext>>,
    watchdog_condvar: Arc<Condvar>,
) -> GameResult<cpal::Stream>
where
    T: cpal::SizedSample + cpal::FromSample<u16>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let buf_size = sample_rate as usize * 10 / 1000;
    let mut bgm_buf = vec![0x8080; buf_size * 2];
    let mut pxt_buf = vec![0x8000; buf_size];
    let mut samples = 0;

    {
        let mut ctx = audio_ctx.lock().unwrap();
        ctx.device_changed(sample_rate);
        pxt_buf.copy_from_slice(ctx.pxt_buf.as_slice());
    }
    log::info!("Audio format: {} {}", sample_rate, channels);

    let stream_result = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut ctx = audio_ctx.lock().unwrap();

            loop {
                if ctx.bgm_fadeout && ctx.bgm_vol > 0.0 {
                    ctx.bgm_vol -= 0.02;

                    if ctx.bgm_vol < 0.0 {
                        ctx.bgm_vol = 0.0;
                    }
                }

                match rx.try_recv() {
                    Ok(PlaybackMessage::PlayOrganyaSong(song)) => {
                        if ctx.state == PlaybackState::Stopped {
                            ctx.saved_state = PlaybackStateType::None;
                        }

                        if ctx.bgm_fadeout {
                            ctx.bgm_fadeout = false;
                            ctx.bgm_vol = ctx.bgm_vol_saved;
                        }

                        ctx.org_engine.start_song(*song, &bank);

                        for i in 0..samples {
                            bgm_buf[i] = 0x8000;
                            ctx.bgm_buf[i] = 0x8000;
                        }
                        samples = ctx.org_engine.render_to(&mut bgm_buf);
                        ctx.bgm_buf.copy_from_slice(bgm_buf.as_slice());
                        ctx.bgm_index = 0;

                        ctx.state = PlaybackState::PlayingOrg;
                    }
                    #[cfg(feature = "ogg-playback")]
                    Ok(PlaybackMessage::PlayOggSongSinglePart(data)) => {
                        if ctx.state == PlaybackState::Stopped {
                            ctx.saved_state = PlaybackStateType::None;
                        }

                        if ctx.bgm_fadeout {
                            ctx.bgm_fadeout = false;
                            ctx.bgm_vol = ctx.bgm_vol_saved;
                        }

                        ctx.ogg_engine.start_single(data);

                        for i in 0..samples {
                            bgm_buf[i] = 0x8000;
                            ctx.bgm_buf[i] = 0x8000;
                        }
                        samples = ctx.ogg_engine.render_to(&mut bgm_buf);
                        ctx.bgm_buf.copy_from_slice(bgm_buf.as_slice());
                        ctx.bgm_index = 0;

                        ctx.state = PlaybackState::PlayingOgg;
                    }
                    #[cfg(feature = "ogg-playback")]
                    Ok(PlaybackMessage::PlayOggSongMultiPart(data_intro, data_loop)) => {
                        if ctx.state == PlaybackState::Stopped {
                            ctx.saved_state = PlaybackStateType::None;
                        }

                        if ctx.bgm_fadeout {
                            ctx.bgm_fadeout = false;
                            ctx.bgm_vol = ctx.bgm_vol_saved;
                        }

                        ctx.ogg_engine.start_multi(data_intro, data_loop);

                        for i in 0..ctx.samples {
                            bgm_buf[i] = 0x8000;
                            ctx.bgm_buf[i] = 0x8000;
                        }
                        ctx.samples = ctx.ogg_engine.render_to(&mut bgm_buf);
                        ctx.bgm_buf.copy_from_slice(bgm_buf.as_slice());
                        ctx.bgm_index = 0;

                        ctx.state = PlaybackState::PlayingOgg;
                    }
                    Ok(PlaybackMessage::PlaySample(id)) => {
                        ctx.pixtone.play_sfx(id);
                    }

                    Ok(PlaybackMessage::LoopSample(id)) => {
                        ctx.pixtone.loop_sfx(id);
                    }
                    Ok(PlaybackMessage::LoopSampleFreq(id, freq)) => {
                        ctx.pixtone.loop_sfx_freq(id, freq);
                    }
                    Ok(PlaybackMessage::StopSample(id)) => {
                        ctx.pixtone.stop_sfx(id);
                    }
                    Ok(PlaybackMessage::Stop) => {
                        if ctx.state == PlaybackState::Stopped {
                            ctx.saved_state = PlaybackStateType::None;
                        }

                        ctx.state = PlaybackState::Stopped;
                    }
                    Ok(PlaybackMessage::SetSpeed(new_speed)) => {
                        assert!(new_speed > 0.0);
                        ctx.speed = new_speed;
                        #[cfg(feature = "ogg-playback")]
                        ctx.ogg_engine.set_sample_rate((sample_rate / new_speed) as usize);
                        ctx.org_engine.set_sample_rate((sample_rate / new_speed) as usize);
                    }
                    Ok(PlaybackMessage::SetSongVolume(new_volume)) => {
                        assert!(ctx.bgm_vol >= 0.0);
                        if ctx.bgm_fadeout {
                            ctx.bgm_vol_saved = new_volume;
                        } else {
                            ctx.bgm_vol = new_volume;
                        }
                    }
                    Ok(PlaybackMessage::SetSampleVolume(new_volume)) => {
                        assert!(ctx.sfx_vol >= 0.0);
                        ctx.sfx_vol = new_volume;
                    }
                    Ok(PlaybackMessage::FadeoutSong) => {
                        ctx.bgm_fadeout = true;
                        ctx.bgm_vol_saved = ctx.bgm_vol;
                    }
                    Ok(PlaybackMessage::SaveState) => {
                        ctx.saved_state = match ctx.state {
                            PlaybackState::Stopped => PlaybackStateType::None,
                            PlaybackState::PlayingOrg => PlaybackStateType::Organya(ctx.org_engine.get_state()),
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => PlaybackStateType::Ogg(ctx.ogg_engine.get_state()),
                        };
                    }
                    Ok(PlaybackMessage::RestoreState) => {
                        let saved_state_loc = std::mem::take(&mut ctx.saved_state);

                        match saved_state_loc {
                            PlaybackStateType::None => {
                                ctx.state = PlaybackState::Stopped;
                            }
                            PlaybackStateType::Organya(playback_state) => {
                                ctx.org_engine.set_state(playback_state, &bank);

                                if ctx.state == PlaybackState::Stopped {
                                    ctx.org_engine.rewind();
                                }

                                for i in 0..samples {
                                    bgm_buf[i] = 0x8000;
                                    ctx.bgm_buf[i] = 0x8000;
                                }

                                // We can't borrow `ctx` as mutable more then once at a time, so we render the samples in `bgm_buf`
                                // and then copy them into `ctx.bgm_buf`. This will allow us to continue a playback, if the audio device is reconnected
                                samples = ctx.org_engine.render_to(&mut bgm_buf);
                                ctx.bgm_buf.copy_from_slice(bgm_buf.as_slice());
                                ctx.bgm_index = 0;

                                if ctx.bgm_fadeout {
                                    ctx.bgm_fadeout = false;
                                    ctx.bgm_vol = ctx.bgm_vol_saved;
                                }

                                ctx.state = PlaybackState::PlayingOrg;
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackStateType::Ogg(playback_state) => {
                                ctx.ogg_engine.set_state(playback_state);

                                if ctx.state == PlaybackState::Stopped {
                                    ctx.ogg_engine.rewind();
                                }

                                for i in 0..samples {
                                    bgm_buf[i] = 0x8000;
                                    ctx.bgm_buf[i] = 0x8000;
                                }
                                samples = ctx.ogg_engine.render_to(&mut bgm_buf);
                                ctx.bgm_buf.copy_from_slice(bgm_buf.as_slice());
                                ctx.bgm_index = 0;

                                if ctx.bgm_fadeout {
                                    ctx.bgm_fadeout = false;
                                    ctx.bgm_vol = ctx.bgm_vol_saved;
                                }

                                ctx.state = PlaybackState::PlayingOgg;
                            }
                        }
                    }
                    Ok(PlaybackMessage::SetSampleParams(id, params)) => {
                        ctx.pixtone.set_sample_parameters(id, params);
                    }
                    Ok(PlaybackMessage::SetOrgInterpolation(interpolation)) => {
                        ctx.org_engine.interpolation = interpolation;
                    }
                    Ok(PlaybackMessage::SetSampleData(id, data)) => {
                        ctx.pixtone.set_sample_data(id, data);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            for frame in data.chunks_mut(channels) {
                let (bgm_sample_l, bgm_sample_r): (u16, u16) = {
                    if ctx.state == PlaybackState::Stopped {
                        (0x8000, 0x8000)
                    } else if ctx.bgm_index < ctx.samples {
                        let samples = (bgm_buf[ctx.bgm_index], bgm_buf[ctx.bgm_index + 1]);
                        ctx.bgm_index += 2;
                        samples
                    } else {
                        for i in 0..ctx.samples {
                            bgm_buf[i] = 0x8000;
                            ctx.bgm_buf[i] = 0x8000;
                        }

                        match ctx.state {
                            PlaybackState::PlayingOrg => {
                                ctx.samples = ctx.org_engine.render_to(&mut bgm_buf);
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => {
                                ctx.samples = ctx.ogg_engine.render_to(&mut bgm_buf);
                            }
                            _ => unreachable!(),
                        }

                        ctx.bgm_buf.copy_from_slice(bgm_buf.as_slice());
                        ctx.bgm_index = 2;
                        (bgm_buf[0], bgm_buf[1])
                    }
                };

                let pxt_sample: u16 = pxt_buf[ctx.pxt_index];

                if ctx.pxt_index < (pxt_buf.len() - 1) {
                    ctx.pxt_index += 1;
                } else {
                    ctx.pxt_index = 0;
                    pxt_buf.fill(0x8000);
                    ctx.pxt_buf.fill(0x8000);

                    let speed = ctx.speed; // We can't make an immutable borrow(ctx.speed) after the mutable(ctx.pixtone)'
                    ctx.pixtone.mix(&mut pxt_buf, sample_rate / speed);
                    ctx.pxt_buf.copy_from_slice(pxt_buf.as_slice());
                }

                if frame.len() >= 2 {
                    let sample_l = clamp(
                        (((bgm_sample_l ^ 0x8000) as i16) as f32 * ctx.bgm_vol) as isize
                            + (((pxt_sample ^ 0x8000) as i16) as f32 * ctx.sfx_vol) as isize,
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;
                    let sample_r = clamp(
                        (((bgm_sample_r ^ 0x8000) as i16) as f32 * ctx.bgm_vol) as isize
                            + (((pxt_sample ^ 0x8000) as i16) as f32 * ctx.sfx_vol) as isize,
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;

                    frame[0] = T::from_sample(sample_l);
                    frame[1] = T::from_sample(sample_r);
                } else {
                    let sample = clamp(
                        ((((bgm_sample_l ^ 0x8000) as i16) + ((bgm_sample_r ^ 0x8000) as i16)) as f32 * ctx.bgm_vol
                            / 2.0) as isize
                            + (((pxt_sample ^ 0x8000) as i16) as f32 * ctx.sfx_vol) as isize,
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;

                    frame[0] = T::from_sample(sample);
                }
            }
        },
        move |err| {
            let _ = err_tx.send(err);
            watchdog_condvar.notify_one();
        },
        None,
    );

    if stream_result.is_err() {
        return Err(GameError::AudioError(stream_result.err().unwrap().to_string()));
    }

    let stream = stream_result.unwrap();
    let _ = stream.play();

    Ok(stream)
}
