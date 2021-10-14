use std::io;
use std::io::{BufRead, BufReader, Lines};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
#[cfg(feature = "ogg-playback")]
use lewton::inside_ogg::OggStreamReader;
use num_traits::clamp;

use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameError::{AudioError, InvalidValue};
use crate::framework::error::{GameError, GameResult};
use crate::framework::filesystem;
use crate::framework::filesystem::File;
use crate::settings::Settings;
#[cfg(feature = "ogg-playback")]
use crate::sound::ogg_playback::{OggPlaybackEngine, SavedOggPlaybackState};
use crate::sound::org_playback::{OrgPlaybackEngine, SavedOrganyaPlaybackState};
use crate::sound::organya::Song;
use crate::sound::pixtone::{PixToneParameters, PixTonePlayback};
use crate::sound::wave_bank::SoundBank;
use crate::str;

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
    tx: Sender<PlaybackMessage>,
    prev_song_id: usize,
    current_song_id: usize,
    no_audio: bool,
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
        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();

        if ctx.headless {
            log::info!("Running in headless mode, skipping initialization.");

            return Ok(SoundManager { tx: tx.clone(), prev_song_id: 0, current_song_id: 0, no_audio: true });
        }

        let host = cpal::default_host();
        let device =
            host.default_output_device().ok_or_else(|| AudioError(str!("Error initializing audio device.")))?;
        let config = device.default_output_config()?;

        let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/organya-wavetable-doukutsu.bin")?)?;

        std::thread::spawn(move || {
            if let Err(err) = match config.sample_format() {
                cpal::SampleFormat::F32 => run::<f32>(rx, bnk, &device, &config.into()),
                cpal::SampleFormat::I16 => run::<i16>(rx, bnk, &device, &config.into()),
                cpal::SampleFormat::U16 => run::<u16>(rx, bnk, &device, &config.into()),
            } {
                log::error!("Something went wrong in audio thread: {}", err);
            }
        });

        Ok(SoundManager { tx: tx.clone(), prev_song_id: 0, current_song_id: 0, no_audio: false })
    }

    pub fn play_sfx(&self, id: u8) {
        if self.no_audio {
            return;
        }
        let _ = self.tx.send(PlaybackMessage::PlaySample(id));
    }

    pub fn loop_sfx(&self, id: u8) {
        if self.no_audio {
            return;
        }
        let _ = self.tx.send(PlaybackMessage::LoopSample(id));
    }

    pub fn stop_sfx(&self, id: u8) {
        if self.no_audio {
            return;
        }
        let _ = self.tx.send(PlaybackMessage::StopSample(id));
    }

    pub fn set_org_interpolation(&self, interpolation: InterpolationMode) {
        if self.no_audio {
            return;
        }
        let _ = self.tx.send(PlaybackMessage::SetOrgInterpolation(interpolation));
    }

    pub fn play_song(
        &mut self,
        song_id: usize,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
    ) -> GameResult {
        if self.current_song_id == song_id || self.no_audio {
            return Ok(());
        }

        if song_id == 0 {
            log::info!("Stopping BGM");

            self.prev_song_id = self.current_song_id;
            self.current_song_id = 0;

            self.tx.send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation))?;
            self.tx.send(PlaybackMessage::SaveState)?;
            self.tx.send(PlaybackMessage::Stop)?;
        } else if let Some(song_name) = constants.music_table.get(song_id) {
            let mut paths = constants.organya_paths.clone();

            paths.insert(0, "/Soundtracks/".to_owned() + &settings.soundtrack + "/");

            if let Some(soundtrack) = constants.soundtracks.get(&settings.soundtrack) {
                paths.insert(0, soundtrack.clone());
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

                            match filesystem::open(ctx, path).map(|f| organya::Song::load_from(f)) {
                                Ok(Ok(org)) => {
                                    log::info!("Playing Organya BGM: {} {}", song_id, path);

                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    self.tx
                                        .send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation))?;
                                    self.tx.send(PlaybackMessage::SaveState)?;
                                    self.tx.send(PlaybackMessage::PlayOrganyaSong(Box::new(org)))?;

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
                                    self.tx.send(PlaybackMessage::SaveState)?;
                                    self.tx.send(PlaybackMessage::PlayOggSongSinglePart(Box::new(song)))?;

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
                                    self.tx.send(PlaybackMessage::SaveState)?;
                                    self.tx.send(PlaybackMessage::PlayOggSongMultiPart(
                                        Box::new(song_intro),
                                        Box::new(song_loop),
                                    ))?;

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

        self.tx.send(PlaybackMessage::SaveState)?;
        self.prev_song_id = self.current_song_id;

        Ok(())
    }

    pub fn restore_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.tx.send(PlaybackMessage::RestoreState)?;
        self.current_song_id = self.prev_song_id;

        Ok(())
    }

    pub fn set_speed(&self, speed: f32) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        if speed <= 0.0 {
            return Err(InvalidValue(str!("Speed must be bigger than 0.0!")));
        }

        self.tx.send(PlaybackMessage::SetSpeed(speed))?;

        Ok(())
    }

    pub fn current_song(&self) -> usize {
        self.current_song_id
    }

    pub fn set_sample_params_from_file<R: io::Read>(&self, id: u8, data: R) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        let mut reader = BufReader::new(data).lines();
        let mut params = PixToneParameters::empty();

        fn next_string<T: FromStr, R: io::Read>(reader: &mut Lines<BufReader<R>>) -> GameResult<T> {
            loop {
                if let Some(Ok(str)) = reader.next() {
                    let str = str.trim();
                    if str == "" || str.starts_with("#") {
                        continue;
                    }

                    let mut splits = str.split(":");

                    let _ = splits.next();
                    if let Some(str) = splits.next() {
                        return str.trim().parse::<T>().map_err(|_| {
                            GameError::ParseError("failed to parse the value as specified type.".to_string())
                        });
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            return Err(GameError::ParseError("unexpected end.".to_string()));
        }

        for channel in params.channels.iter_mut() {
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

    pub fn set_sample_params(&self, id: u8, params: PixToneParameters) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.tx.send(PlaybackMessage::SetSampleParams(id, params))?;

        Ok(())
    }
}

enum PlaybackMessage {
    Stop,
    PlayOrganyaSong(Box<Song>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongSinglePart(Box<OggStreamReader<File>>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongMultiPart(Box<OggStreamReader<File>>, Box<OggStreamReader<File>>),
    PlaySample(u8),
    LoopSample(u8),
    StopSample(u8),
    SetSpeed(f32),
    SaveState,
    RestoreState,
    SetSampleParams(u8, PixToneParameters),
    SetOrgInterpolation(InterpolationMode),
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

fn run<T>(
    rx: Receiver<PlaybackMessage>,
    bank: SoundBank,
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> GameResult
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let mut state = PlaybackState::Stopped;
    let mut saved_state: PlaybackStateType = PlaybackStateType::None;
    let mut speed = 1.0;
    let mut org_engine = OrgPlaybackEngine::new();
    #[cfg(feature = "ogg-playback")]
    let mut ogg_engine = OggPlaybackEngine::new();
    let mut pixtone = PixTonePlayback::new();
    pixtone.create_samples();

    log::info!("Audio format: {} {}", sample_rate, channels);
    org_engine.set_sample_rate(sample_rate as usize);
    #[cfg(feature = "ogg-playback")]
    {
        org_engine.loops = usize::MAX;
        ogg_engine.set_sample_rate(sample_rate as usize);
    }

    let buf_size = sample_rate as usize * 10 / 1000;
    let mut bgm_buf = vec![0x8080; buf_size * 2];
    let mut pxt_buf = vec![0x8000; buf_size];
    let mut bgm_index = 0;
    let mut pxt_index = 0;
    let mut samples = 0;
    pixtone.mix(&mut pxt_buf, sample_rate);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            loop {
                match rx.try_recv() {
                    Ok(PlaybackMessage::PlayOrganyaSong(song)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        org_engine.start_song(*song, &bank);

                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8080
                        }
                        samples = org_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::PlayingOrg;
                    }
                    #[cfg(feature = "ogg-playback")]
                    Ok(PlaybackMessage::PlayOggSongSinglePart(data)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        ogg_engine.start_single(data);

                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }
                        samples = ogg_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::PlayingOgg;
                    }
                    #[cfg(feature = "ogg-playback")]
                    Ok(PlaybackMessage::PlayOggSongMultiPart(data_intro, data_loop)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        ogg_engine.start_multi(data_intro, data_loop);

                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }
                        samples = ogg_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::PlayingOgg;
                    }
                    Ok(PlaybackMessage::PlaySample(id)) => {
                        pixtone.play_sfx(id);
                    }

                    Ok(PlaybackMessage::LoopSample(id)) => {
                        pixtone.loop_sfx(id);
                    }
                    Ok(PlaybackMessage::StopSample(id)) => {
                        pixtone.stop_sfx(id);
                    }
                    Ok(PlaybackMessage::Stop) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        state = PlaybackState::Stopped;
                    }
                    Ok(PlaybackMessage::SetSpeed(new_speed)) => {
                        assert!(new_speed > 0.0);
                        speed = new_speed;
                        #[cfg(feature = "ogg-playback")]
                        ogg_engine.set_sample_rate((sample_rate / new_speed) as usize);
                        org_engine.set_sample_rate((sample_rate / new_speed) as usize);
                    }
                    Ok(PlaybackMessage::SaveState) => {
                        saved_state = match state {
                            PlaybackState::Stopped => PlaybackStateType::None,
                            PlaybackState::PlayingOrg => PlaybackStateType::Organya(org_engine.get_state()),
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => PlaybackStateType::Ogg(ogg_engine.get_state()),
                        };
                    }
                    Ok(PlaybackMessage::RestoreState) => {
                        let mut saved_state_loc = PlaybackStateType::None;
                        std::mem::swap(&mut saved_state_loc, &mut saved_state);

                        match saved_state_loc {
                            PlaybackStateType::None => {}
                            PlaybackStateType::Organya(playback_state) => {
                                org_engine.set_state(playback_state, &bank);

                                if state == PlaybackState::Stopped {
                                    org_engine.rewind();
                                }

                                for i in &mut bgm_buf[0..samples] {
                                    *i = 0x8080
                                }
                                samples = org_engine.render_to(&mut bgm_buf);
                                bgm_index = 0;

                                state = PlaybackState::PlayingOrg;
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackStateType::Ogg(playback_state) => {
                                ogg_engine.set_state(playback_state);

                                if state == PlaybackState::Stopped {
                                    ogg_engine.rewind();
                                }

                                for i in &mut bgm_buf[0..samples] {
                                    *i = 0x8000
                                }
                                samples = ogg_engine.render_to(&mut bgm_buf);
                                bgm_index = 0;

                                state = PlaybackState::PlayingOgg;
                            }
                        }
                    }
                    Ok(PlaybackMessage::SetSampleParams(id, params)) => {
                        pixtone.set_sample_parameters(id, params);
                    }
                    Ok(PlaybackMessage::SetOrgInterpolation(interpolation)) => {
                        org_engine.interpolation = interpolation;
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            for frame in data.chunks_mut(channels) {
                let (bgm_sample_l, bgm_sample_r): (u16, u16) = {
                    if state == PlaybackState::Stopped {
                        (0x8000, 0x8000)
                    } else if bgm_index < samples {
                        match state {
                            PlaybackState::PlayingOrg => {
                                let sample = bgm_buf[bgm_index];
                                bgm_index += 1;
                                ((sample & 0xff) << 8, sample & 0xff00)
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => {
                                let samples = (bgm_buf[bgm_index], bgm_buf[bgm_index + 1]);
                                bgm_index += 2;
                                samples
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8080
                        }

                        match state {
                            PlaybackState::PlayingOrg => {
                                samples = org_engine.render_to(&mut bgm_buf);
                                bgm_index = 1;
                                let sample = bgm_buf[0];
                                ((sample & 0xff) << 8, sample & 0xff00)
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => {
                                samples = ogg_engine.render_to(&mut bgm_buf);
                                bgm_index = 2;
                                (bgm_buf[0], bgm_buf[1])
                            }
                            _ => unreachable!(),
                        }
                    }
                };

                let pxt_sample: u16 = pxt_buf[pxt_index];

                if pxt_index < (pxt_buf.len() - 1) {
                    pxt_index += 1;
                } else {
                    pxt_index = 0;
                    for i in pxt_buf.iter_mut() {
                        *i = 0x8000
                    }
                    pixtone.mix(&mut pxt_buf, sample_rate / speed);
                }

                if frame.len() >= 2 {
                    let sample_l = clamp(
                        (((bgm_sample_l ^ 0x8000) as i16) as isize) + (((pxt_sample ^ 0x8000) as i16) as isize),
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;
                    let sample_r = clamp(
                        (((bgm_sample_r ^ 0x8000) as i16) as isize) + (((pxt_sample ^ 0x8000) as i16) as isize),
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;

                    frame[0] = Sample::from::<u16>(&sample_l);
                    frame[1] = Sample::from::<u16>(&sample_r);
                } else {
                    let sample = clamp(
                        ((((bgm_sample_l ^ 0x8000) as i16) + ((bgm_sample_r ^ 0x8000) as i16)) / 2) as isize
                            + (((pxt_sample ^ 0x8000) as i16) as isize),
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;

                    frame[0] = Sample::from::<u16>(&sample);
                }
            }
        },
        err_fn,
    )?;

    stream.play()?;

    let mut saved_state = true;
    loop {
        std::thread::sleep(Duration::from_millis(10));

        {
            let mutex = crate::GAME_SUSPENDED.lock().unwrap();
            let state = *mutex;
            if saved_state != state {
                saved_state = state;

                if state {
                    if let Err(e) = stream.pause() {
                        log::error!("Failed to pause the stream: {:?}", e);
                    }
                } else {
                    if let Err(e) = stream.play() {
                        log::error!("Failed to unpause the stream: {:?}", e);
                    }
                }
            }
        }
    }
}
