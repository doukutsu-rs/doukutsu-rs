#pragma once

#include "engine_constants.h"

namespace doukutsu_rs::caret
{
    class CaretType
    {
    public:
        enum Value
        {
            None,
            Bubble,
            ProjectileDissipation,
            Shoot,
            SnakeAfterimage,
            Zzz,
            SnakeAfterimage2,
            Exhaust,
            DrownedQuote,
            QuestionMark,
            LevelUp,
            HurtParticles,
            Explosion,
            LittleParticles,
            Unknown,
            SmallProjectileDissipation,
            EmptyText,
            PushJumpKey,
        };

        CaretType() = default;
        constexpr CaretType(Value val) : value(val) {}

        constexpr operator Value() const { return value; }
        explicit operator bool() = delete;

        static constexpr std::optional<CaretType> from_int(int val)
        {
            return val == 0
                       ? std::make_optional(CaretType(None))
                   : val == 1  ? std::make_optional(CaretType(Bubble))
                   : val == 2  ? std::make_optional(CaretType(ProjectileDissipation))
                   : val == 3  ? std::make_optional(CaretType(Shoot))
                   : val == 4  ? std::make_optional(CaretType(SnakeAfterimage))
                   : val == 5  ? std::make_optional(CaretType(Zzz))
                   : val == 6  ? std::make_optional(CaretType(SnakeAfterimage2))
                   : val == 7  ? std::make_optional(CaretType(Exhaust))
                   : val == 8  ? std::make_optional(CaretType(DrownedQuote))
                   : val == 9  ? std::make_optional(CaretType(QuestionMark))
                   : val == 10 ? std::make_optional(CaretType(LevelUp))
                   : val == 11 ? std::make_optional(CaretType(HurtParticles))
                   : val == 12 ? std::make_optional(CaretType(Explosion))
                   : val == 13 ? std::make_optional(CaretType(LittleParticles))
                   : val == 14 ? std::make_optional(CaretType(Unknown))
                   : val == 15 ? std::make_optional(CaretType(SmallProjectileDissipation))
                   : val == 16 ? std::make_optional(CaretType(EmptyText))
                   : val == 17 ? std::make_optional(CaretType(PushJumpKey))
                               : std::nullopt;
        }

    private:
        Value value;
    };

    class Caret
    {
    public:
        CaretType ctype;
        int32_t x;
        int32_t y;
        int32_t vel_x;
        int32_t vel_y;
        int32_t offset_x;
        int32_t offset_y;
        int32_t prev_x;
        int32_t prev_y;
        common::Condition cond;
        common::Direction direction;
        common::Rect<uint16_t> anim_rect;

    private:
        uint16_t action_num;
        uint16_t anim_num;
        uint16_t anim_counter;

    public:
        //     pub fn new(x: i32, y: i32, ctype: CaretType, direct: Direction, constants: &EngineConstants) -> Caret {
        //     let (offset_x, offset_y) = constants.caret.offsets[ctype as usize];

        //     Caret {
        //         ctype,
        //         x,
        //         y,
        //         vel_x: 0,
        //         vel_y: 0,
        //         offset_x,
        //         offset_y,
        //         prev_x: x,
        //         prev_y: y,
        //         cond: Condition(0x80),
        //         direction: direct,
        //         anim_rect: Rect::new(0, 0, 0, 0),
        //         action_num: 0,
        //         anim_num: 0,
        //         anim_counter: 0,
        //     }
        // }
        Caret(int32_t x, int32_t y, CaretType ctype, common::Direction direction, const engine_constants::EngineConstants &constants)
            : ctype(ctype),
              x(x),
              y(y),
              vel_x(0),
              vel_y(0),
              prev_x(x),
              prev_y(y),
              cond(common::Condition(0x80)),
              direction(direction),
              anim_rect(common::Rect<uint16_t>{0, 0, 0, 0}),
              action_num(0),
              anim_num(0),
              anim_counter(0)
        {
            auto [offset_x, offset_y] = constants.caret.offsets[(int)ctype];

            this->offset_x = offset_x;
            this->offset_y = offset_y;
        }
    };
};