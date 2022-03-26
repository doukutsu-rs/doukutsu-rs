#pragma once

#include <cstdint>
#include <vector>
#include <string>

#include "common.h"
#include "rng.h"
#include "caret.h"
#include "npc.h"
#include "input/touch_controls.h"
#include "sound/sound_manager.h"

namespace doukutsu_rs::shared_game_state
{
    class TimingMode
    {
    public:
        enum Value
        {
            _50Hz,
            _60Hz,
            FrameSynchronized,
        };

        TimingMode() = default;
        TimingMode(Value value) : value(value) {}

        constexpr operator Value() const { return value; }
        constexpr operator bool() = delete;

        constexpr bool operator==(const TimingMode &other) const { return value == other.value; }
        constexpr bool operator!=(const TimingMode &other) const { return value != other.value; }

        constexpr bool operator==(Value other) const { return value == other; }
        constexpr bool operator!=(Value other) const { return value != other; }

        constexpr uintptr_t get_delta() const
        {
            switch (value)
            {
            case Value::_50Hz:
                return 1000000000 / 50;
            case Value::_60Hz:
                return 1000000000 / 60;
            case Value::FrameSynchronized:
                return 0;
            }
        }

        constexpr double get_delta_millis() const
        {
            switch (value)
            {
            case Value::_50Hz:
                return 1000.0 / 50.0;
            case Value::_60Hz:
                return 1000.0 / 60.0;
            case Value::FrameSynchronized:
                return 0.0;
            }
        }

        constexpr uintptr_t get_tps() const
        {
            switch (value)
            {
            case Value::_50Hz:
                return 50;
            case Value::_60Hz:
                return 60;
            case Value::FrameSynchronized:
                return 0;
            }
        }

    private:
        Value value;
    };

    class TileSize
    {
    public:
        enum Value
        {
            Tile8x8,
            Tile16x16,
        };

        TileSize() = default;
        TileSize(Value value) : value(value) {}

        constexpr operator Value() const { return value; }
        constexpr operator bool() = delete;

        constexpr bool operator==(const TileSize &other) const { return value == other.value; }
        constexpr bool operator!=(const TileSize &other) const { return value != other.value; }

        constexpr bool operator==(Value other) const { return value == other; }
        constexpr bool operator!=(Value other) const { return value != other; }

        constexpr operator float() const { return as_float(); }
        constexpr operator int() const { return as_int(); }

        constexpr float as_float() const
        {
            switch (value)
            {
            case Value::Tile8x8:
                return 8.0f;
            case Value::Tile16x16:
                return 16.0f;
            default:
                common::unreachable();
            }
        }

        constexpr int as_int() const
        {
            switch (value)
            {
            case Value::Tile8x8:
                return 8;
            case Value::Tile16x16:
                return 16;
            default:
                common::unreachable();
            }
        }

    private:
        Value value;
    };

    // pub struct SharedGameState {
    //     pub control_flags: ControlFlags,
    //     pub game_flags: BitVec,
    //     pub skip_flags: BitVec,
    //     pub map_flags: BitVec,
    //     pub fade_state: FadeState,
    //     /// RNG used by game state, using it for anything else might cause unintended side effects and break replays.
    //     pub game_rng: XorShift,
    //     /// RNG used by graphics effects that aren't dependent on game's state.
    //     pub effect_rng: XorShift,
    //     pub tile_size: TileSize,
    //     pub quake_counter: u16,
    //     pub super_quake_counter: u16,
    //     pub teleporter_slots: Vec<(u16, u16)>,
    //     pub carets: Vec<Caret>,
    //     pub touch_controls: TouchControls,
    //     pub mod_path: Option<String>,
    //     pub mod_list: ModList,
    //     pub npc_table: NPCTable,
    //     pub npc_super_pos: (i32, i32),
    //     pub npc_curly_target: (i32, i32),
    //     pub npc_curly_counter: u16,
    //     pub water_level: i32,
    //     pub stages: Vec<StageData>,
    //     pub frame_time: f64,
    //     pub debugger: bool,
    //     pub scale: f32,
    //     pub canvas_size: (f32, f32),
    //     pub screen_size: (f32, f32),
    //     pub preferred_viewport_size: (f32, f32),
    //     pub next_scene: Option<Box<dyn Scene>>,
    //     pub textscript_vm: TextScriptVM,
    //     pub creditscript_vm: CreditScriptVM,
    //     pub lightmap_canvas: Option<Box<dyn BackendTexture>>,
    //     pub season: Season,
    //     pub menu_character: MenuCharacter,
    //     pub constants: EngineConstants,
    //     pub font: BMFontRenderer,
    //     pub texture_set: TextureSet,
    //     #[cfg(feature = "scripting-lua")]
    //     pub lua: LuaScriptingState,
    //     pub sound_manager: SoundManager,
    //     pub settings: Settings,
    //     pub save_slot: usize,
    //     pub difficulty: GameDifficulty,
    //     pub replay_state: ReplayState,
    //     pub mod_requirements: ModRequirements,
    //     pub shutdown: bool,
    // }

    class SharedGameState
    {
    public:
        common::ControlFlags control_flags;
        std::vector<bool> game_flags;
        std::vector<bool> skip_flags;
        std::vector<bool> map_flags;
        common::FadeState fade_state;
        rng::XorShift game_rng;
        rng::XorShift effect_rng;
        TileSize tile_size;
        uint16_t quake_counter;
        uint16_t super_quake_counter;
        std::vector<std::pair<uint16_t, uint16_t>> teleporter_slots;
        std::vector<caret::Caret> carets;
        input::touch_controls::TouchControls touch_controls;
        std::optional<std::string> mod_path;
        // mod_list::ModList mod_list;
        npc::NPCTable npc_table;
        std::pair<int32_t, int32_t> npc_super_pos;
        std::pair<int32_t, int32_t> npc_curly_target;
        uint16_t npc_curly_counter;
        int32_t water_level;
        // std::vector<stage::StageData> stages;
        double frame_time;
        bool debugger;
        float scale;
        std::pair<float, float> canvas_size;
        std::pair<float, float> screen_size;
        std::pair<float, float> preferred_viewport_size;
        // std::optional<std::shared_ptr<scene::Scene>> next_scene;
        // scripting::text_script::TextScriptVM textscript_vm;
        // scripting::credit_script::CreditScriptVM creditscript_vm;

        sound::SoundManager sound_manager;
        bool shutdown;
    };
};