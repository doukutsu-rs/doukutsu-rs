#include "sound_manager.h"

namespace doukutsu_rs::sound
{

SoundManager::SoundManager(framework::context::Context &ctx) 
    : prev_song_id(0), current_song_id(0), no_audio(false)
{
    // Initialize MPSC channel
    auto [sender, receiver] = mpsc::make_channel<PlaybackMessage>();
    tx = std::make_unique<mpsc::Sender<PlaybackMessage>>(std::move(sender));
    // TODO: Store receiver and initialize sound system properly
}

void SoundManager::pause()
{
    // TODO: Implement pause
}

void SoundManager::resume()
{
    // TODO: Implement resume  
}

void SoundManager::play_sfx(uint8_t sample_id)
{
    // TODO: Implement sound effect playback
    if (!no_audio && tx)
    {
        auto msg = PlaybackMessageType::PlaySample{sample_id};
        tx->send(PlaybackMessage(msg));
    }
}

void SoundManager::play_sfx_freq(uint8_t sample_id, float freq)
{
    // TODO: Implement sound effect playback with frequency
    if (!no_audio && tx)
    {
        auto msg = PlaybackMessageType::LoopSampleFreq{sample_id, freq};
        tx->send(PlaybackMessage(msg));
    }
}

void SoundManager::stop_sfx(uint8_t sample_id)
{
    // TODO: Implement stop sound effect
    if (!no_audio && tx)
    {
        auto msg = PlaybackMessageType::StopSample{sample_id};
        tx->send(PlaybackMessage(msg));
    }
}

void SoundManager::set_org_interpolation(InterpolationMode mode)
{
    // TODO: Implement interpolation mode setting
}

void SoundManager::set_song_volume(float volume)
{
    // TODO: Implement song volume setting
}

void SoundManager::set_sfx_volume(float volume)
{
    // TODO: Implement sfx volume setting
}

void SoundManager::set_sfx_samples(uint8_t id, std::vector<int16_t> data)
{
    // TODO: Implement sfx sample data setting
}

void SoundManager::reload_songs(const engine_constants::EngineConstants &constants,
                               const settings::Settings &settings,
                               framework::context::Context &ctx)
{
    // TODO: Implement song reloading
}

void SoundManager::play_song(uintptr_t song_id, const engine_constants::EngineConstants &constants,
                           const settings::Settings &settings,
                           framework::context::Context &ctx)
{
    // TODO: Implement song playback
    current_song_id = song_id;
}

void SoundManager::save_state()
{
    // TODO: Implement state saving
}

void SoundManager::restore_state()
{
    // TODO: Implement state restoration
}

void SoundManager::set_speed(float speed)
{
    // TODO: Implement speed setting
}

uintptr_t SoundManager::current_song()
{
    return current_song_id;
}

}