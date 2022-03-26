#pragma once

#include "../3rdparty/mpsc_channel.h"
#include <variant>

namespace doukutsu_rs::sound
{
    enum class SongFormat
    {
        Organya,
        OggSinglePart,
        OggMultiPart,
    };

    class Song
    {
    };

    class PixToneParameters
    {
    };

    enum class InterpolationMode
    {
        None,
        Linear,
        Cubic,
    };

    struct PlaybackMessageType
    {
    public:
        struct Stop
        {
        };
        struct PlayOrganyaSong
        {
            std::unique_ptr<Song> song;
        };
        struct PlayOggSongSinglePart
        {
        };
        struct PlayOggSongMultiPart
        {
        };
        struct PlaySample
        {
            uint8_t sample_id;
        };
        struct LoopSample
        {
            uint8_t sample_id;
        };
        struct LoopSampleFreq
        {
            uint8_t sample_id;
            float freq;
        };
        struct StopSample
        {
            uint8_t sample_id;
        };
        struct SetSpeed
        {
            float speed;
        };
        struct SetSongVolume
        {
            float volume;
        };
        struct SetSampleVolume
        {
            float volume;
        };
        struct SaveState
        {
        };
        struct RestoreState
        {
        };
        struct SetSampleParams
        {
            uint8_t sample_id;
            PixToneParameters params;
        };
        struct SetOrgInterpolation
        {
            InterpolationMode mode;
        };
        struct SetSampleData
        {
            uint8_t sample_id;
            std::vector<int16_t> data;
        };
    };

    typedef std::variant<
        PlaybackMessageType::Stop,
        PlaybackMessageType::PlayOrganyaSong,
        PlaybackMessageType::PlayOggSongSinglePart,
        PlaybackMessageType::PlayOggSongMultiPart,
        PlaybackMessageType::PlaySample,
        PlaybackMessageType::LoopSample,
        PlaybackMessageType::LoopSampleFreq,
        PlaybackMessageType::StopSample,
        PlaybackMessageType::SetSpeed,
        PlaybackMessageType::SetSongVolume,
        PlaybackMessageType::SetSampleVolume,
        PlaybackMessageType::SaveState,
        PlaybackMessageType::RestoreState,
        PlaybackMessageType::SetSampleParams,
        PlaybackMessageType::SetOrgInterpolation,
        PlaybackMessageType::SetSampleData>
        PlaybackMessage;

    class SoundManager
    {
    private:
        mpsc::Sender<PlaybackMessage> tx;
        uintptr_t prev_song_id;
        uintptr_t current_song_id;
        bool no_audio;
        // std::optional<cpal::Stream> stream;

    public:
        explicit SoundManager(framework::context::Context &ctx);

        SoundManager(const SoundManager &) = delete;
        SoundManager &operator=(const SoundManager &) = delete;

    public:
    };
};