#pragma once

#include "engine_constants/npcs.h"

#include <array>
#include <vector>

namespace doukutsu_rs::engine_constants
{
    // pub struct CaretConsts {
    //     pub offsets: [(i32, i32); 18],
    //     pub bubble_left_rects: Vec<Rect<u16>>,
    //     pub bubble_right_rects: Vec<Rect<u16>>,
    //     pub projectile_dissipation_left_rects: Vec<Rect<u16>>,
    //     pub projectile_dissipation_right_rects: Vec<Rect<u16>>,
    //     pub projectile_dissipation_up_rects: Vec<Rect<u16>>,
    //     pub shoot_rects: Vec<Rect<u16>>,
    //     pub zzz_rects: Vec<Rect<u16>>,
    //     pub drowned_quote_left_rect: Rect<u16>,
    //     pub drowned_quote_right_rect: Rect<u16>,
    //     pub level_up_rects: Vec<Rect<u16>>,
    //     pub level_down_rects: Vec<Rect<u16>>,
    //     pub hurt_particles_rects: Vec<Rect<u16>>,
    //     pub explosion_rects: Vec<Rect<u16>>,
    //     pub little_particles_rects: Vec<Rect<u16>>,
    //     pub exhaust_rects: Vec<Rect<u16>>,
    //     pub question_left_rect: Rect<u16>,
    //     pub question_right_rect: Rect<u16>,
    //     pub small_projectile_dissipation: Vec<Rect<u16>>,
    //     pub empty_text: Vec<Rect<u16>>,
    //     pub push_jump_key: Vec<Rect<u16>>,
    // }
    struct CaretConsts
    {
        std::array<std::pair<int32_t, int32_t>, 18> offsets;
        std::vector<common::Rect<uint16_t>> bubble_left_rects;
        std::vector<common::Rect<uint16_t>> bubble_right_rects;
        std::vector<common::Rect<uint16_t>> projectile_dissipation_left_rects;
        std::vector<common::Rect<uint16_t>> projectile_dissipation_right_rects;
        std::vector<common::Rect<uint16_t>> projectile_dissipation_up_rects;
        std::vector<common::Rect<uint16_t>> shoot_rects;
        std::vector<common::Rect<uint16_t>> zzz_rects;
        common::Rect<uint16_t> drowned_quote_left_rect;
        common::Rect<uint16_t> drowned_quote_right_rect;
        std::vector<common::Rect<uint16_t>> level_up_rects;
        std::vector<common::Rect<uint16_t>> level_down_rects;
        std::vector<common::Rect<uint16_t>> hurt_particles_rects;
        std::vector<common::Rect<uint16_t>> explosion_rects;
        std::vector<common::Rect<uint16_t>> little_particles_rects;
        std::vector<common::Rect<uint16_t>> exhaust_rects;
        common::Rect<uint16_t> question_left_rect;
        common::Rect<uint16_t> question_right_rect;
        std::vector<common::Rect<uint16_t>> small_projectile_dissipation;
        std::vector<common::Rect<uint16_t>> empty_text;
        std::vector<common::Rect<uint16_t>> push_jump_key;
    };

    class EngineConstants
    {
    public:
        CaretConsts caret;
        npcs::NPCConsts npc;
    };
};