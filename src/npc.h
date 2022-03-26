#pragma once

#include <cstdint>
#include <array>

#include "common.h"
#include "rng.h"

namespace doukutsu_rs
{
    namespace shared_game_state
    {
        class SharedGameState;
    }

    namespace player
    {
        class Player;
    }
}

namespace doukutsu_rs::npc
{
    constexpr int MAX_FALL_SPEED = 0x5ff;

    struct NPCFlag
    {
        uint16_t value;

        constexpr NPCFlag(uint16_t value) : value(value) {}
        constexpr NPCFlag() : value(0) {}

        constexpr bool solid_soft() const { return value & 0x01; }
        constexpr bool ignore_tile_44() const { return value & 0x02; }
        constexpr bool invulnerable() const { return value & 0x04; }
        constexpr bool ignore_solidity() const { return value & 0x08; }
        constexpr bool bouncy() const { return value & 0x10; }
        constexpr bool shootable() const { return value & 0x20; }
        constexpr bool solid_hard() const { return value & 0x40; }
        constexpr bool rear_and_top_not_hurt() const { return value & 0x80; }
        constexpr bool event_when_touched() const { return value & 0x100; }
        constexpr bool event_when_killed() const { return value & 0x200; }
        constexpr bool flag_x400() const { return value & 0x400; }
        constexpr bool appear_when_flag_set() const { return value & 0x800; }
        constexpr bool spawn_facing_right() const { return value & 0x1000; }
        constexpr bool interactable() const { return value & 0x2000; }
        constexpr bool hide_unless_flag_set() const { return value & 0x4000; }
        constexpr bool show_damage() const { return value & 0x8000; }

        constexpr void set_solid_soft(bool value) { this->value = value ? (this->value | 0x01) : (this->value & ~0x01); }
        constexpr void set_ignore_tile_44(bool value) { this->value = value ? (this->value | 0x02) : (this->value & ~0x02); }
        constexpr void set_invulnerable(bool value) { this->value = value ? (this->value | 0x04) : (this->value & ~0x04); }
        constexpr void set_ignore_solidity(bool value) { this->value = value ? (this->value | 0x08) : (this->value & ~0x08); }
        constexpr void set_bouncy(bool value) { this->value = value ? (this->value | 0x10) : (this->value & ~0x10); }
        constexpr void set_shootable(bool value) { this->value = value ? (this->value | 0x20) : (this->value & ~0x20); }
        constexpr void set_solid_hard(bool value) { this->value = value ? (this->value | 0x40) : (this->value & ~0x40); }
        constexpr void set_rear_and_top_not_hurt(bool value) { this->value = value ? (this->value | 0x80) : (this->value & ~0x80); }
        constexpr void set_event_when_touched(bool value) { this->value = value ? (this->value | 0x100) : (this->value & ~0x100); }
        constexpr void set_event_when_killed(bool value) { this->value = value ? (this->value | 0x200) : (this->value & ~0x200); }
        constexpr void set_flag_x400(bool value) { this->value = value ? (this->value | 0x400) : (this->value & ~0x400); }
        constexpr void set_appear_when_flag_set(bool value) { this->value = value ? (this->value | 0x800) : (this->value & ~0x800); }
        constexpr void set_spawn_facing_right(bool value) { this->value = value ? (this->value | 0x1000) : (this->value & ~0x1000); }
        constexpr void set_interactable(bool value) { this->value = value ? (this->value | 0x2000) : (this->value & ~0x2000); }
        constexpr void set_hide_unless_flag_set(bool value) { this->value = value ? (this->value | 0x4000) : (this->value & ~0x4000); }
        constexpr void set_show_damage(bool value) { this->value = value ? (this->value | 0x8000) : (this->value & ~0x8000); }
    };

    enum class NPCLayer : uint8_t
    {
        Background = 0,
        Middleground = 1,
        Foreground = 2,
    };

    class Players
    {
    };

    class NPCTable
    {
    public:
    };

    class NPCList;

    class NPC
    {
    public:
        uint16_t id;
        uint16_t npc_type;
        int32_t x;
        int32_t y;
        int32_t vel_x;
        int32_t vel_y;
        int32_t vel_x2;
        int32_t vel_y2;
        int32_t target_x;
        int32_t target_y;
        int32_t prev_x;
        int32_t prev_y;
        uint16_t exp;
        NPCLayer layer;
        uint8_t size;
        uint16_t shock;
        uint16_t life;
        uint16_t damage;
        uint16_t spritesheet_id;
        common::Condition cond;
        common::Flag flags;
        NPCFlag npc_flags;
        common::Direction direction;
        uint16_t tsc_direction;
        uint16_t parent_id;
        uint16_t action_num;
        uint16_t anim_num;
        uint16_t flag_num;
        uint16_t event_num;
        uint16_t action_counter;
        uint16_t action_counter2;
        uint16_t action_counter3;
        uint16_t anim_counter;
        common::Rect<uint16_t> anim_rect;
        common::Rect<uint32_t> display_bounds;
        common::Rect<uint32_t> hit_bounds;
        rng::Xoroshiro32PlusPlus rng;
        // NumberPopup popup;

        explicit NPC() : id(0), npc_type(0), x(0), y(0), vel_x(0), vel_y(0), vel_x2(0), vel_y2(0),
                         target_x(0), target_y(0), prev_x(0), prev_y(0), exp(0), layer(NPCLayer::Middleground),
                         size(0), shock(0), life(0), damage(0), spritesheet_id(0), cond(common::Condition(0)),
                         flags(common::Flag(0)), npc_flags(NPCFlag(0)), direction(common::Direction::Left),
                         tsc_direction(0), parent_id(0), action_num(0), anim_num(0), flag_num(0), event_num(0),
                         action_counter(0), action_counter2(0), action_counter3(0), anim_counter(0),
                         anim_rect(common::Rect<uint16_t>{0, 0, 0, 0}), display_bounds(common::Rect<uint32_t>{0, 0, 0, 0}),
                         hit_bounds(common::Rect<uint32_t>{0, 0, 0, 0}), rng(rng::Xoroshiro32PlusPlus(0)) {}

        static NPC create(uint16_t npc_type, NPCTable &table);

        player::Player &get_closest_player_mut(Players &players);
        player::Player const &get_closest_player_ref(Players const &players) const;
        void face_player(const player::Player &player);

        void clamp_fall_speed()
        {
            if (vel_y > MAX_FALL_SPEED)
                vel_y = MAX_FALL_SPEED;
        }

    protected:
#pragma region NPC AI procedures
        void tick_n254_helicopter(shared_game_state::SharedGameState &state, NPCList &npc_list);
        void tick_n255_helicopter_blades(shared_game_state::SharedGameState &state, NPCList &npc_list);
        void tick_n260_shovel_brigade_caged(shared_game_state::SharedGameState &state, NPCList &npc_list);
        void tick_n261_chie_caged(shared_game_state::SharedGameState &state, NPCList &npc_list, Players &players);
        void tick_n262_chaco_caged(shared_game_state::SharedGameState &state, NPCList &npc_list, Players &players);
#pragma endregion
    };

    class NPCList
    {
    private:
        std::array<NPC, 512> npcs;
        uint16_t max_npc;

    public:
        bool spawn(uint16_t min_id, NPC npc);
    };
};