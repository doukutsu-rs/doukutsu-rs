#pragma once

#include <cstdint>
#include <cmath>
#include <optional>
#include <tuple>
#include <concepts>
#include <type_traits>

namespace doukutsu_rs::texture_set
{
    extern float I_MAG;
    extern float G_MAG;
};

namespace doukutsu_rs::common
{
    struct Flag
    {
        uint32_t value;

        constexpr Flag() : value(0) {}
        constexpr Flag(uint32_t value) : value(value) {}

        bool operator==(const Flag &other) const { return value == other.value; }
        bool operator!=(const Flag &other) const { return value != other.value; }

        constexpr bool hit_left_wall() const { return value & 0x01; }
        constexpr bool hit_top_wall() const { return value & 0x02; }
        constexpr bool hit_right_wall() const { return value & 0x04; }
        constexpr bool hit_bottom_wall() const { return value & 0x08; }
        constexpr bool hit_right_slope() const { return value & 0x10; }
        constexpr bool hit_left_slope() const { return value & 0x20; }
        constexpr bool hit_upper_right_slope() const { return value & 0x40; }
        constexpr bool hit_upper_left_slope() const { return value & 0x80; }
        constexpr bool in_water() const { return value & 0x100; }
        constexpr bool weapon_hit_block() const { return value & 0x200; }
        constexpr bool hit_by_spike() const { return value & 0x400; }
        constexpr bool water_splash_facing_right() const { return value & 0x800; }
        constexpr bool force_left() const { return value & 0x1000; }
        constexpr bool force_up() const { return value & 0x2000; }
        constexpr bool force_right() const { return value & 0x4000; }
        constexpr bool force_down() const { return value & 0x8000; }
        constexpr bool hit_left_higher_half() const { return value & 0x10000; }
        constexpr bool hit_left_lower_half() const { return value & 0x20000; }
        constexpr bool hit_right_lower_half() const { return value & 0x40000; }
        constexpr bool hit_right_higher_half() const { return value & 0x80000; }

        constexpr void set_hit_left_wall(bool value) { this->value = value ? (this->value | 0x01) : (this->value & ~0x01); }
        constexpr void set_hit_top_wall(bool value) { this->value = value ? (this->value | 0x02) : (this->value & ~0x02); }
        constexpr void set_hit_right_wall(bool value) { this->value = value ? (this->value | 0x04) : (this->value & ~0x04); }
        constexpr void set_hit_bottom_wall(bool value) { this->value = value ? (this->value | 0x08) : (this->value & ~0x08); }
        constexpr void set_hit_right_slope(bool value) { this->value = value ? (this->value | 0x10) : (this->value & ~0x10); }
        constexpr void set_hit_left_slope(bool value) { this->value = value ? (this->value | 0x20) : (this->value & ~0x20); }
        constexpr void set_hit_upper_right_slope(bool value) { this->value = value ? (this->value | 0x40) : (this->value & ~0x40); }
        constexpr void set_hit_upper_left_slope(bool value) { this->value = value ? (this->value | 0x80) : (this->value & ~0x80); }
        constexpr void set_in_water(bool value) { this->value = value ? (this->value | 0x100) : (this->value & ~0x100); }
        constexpr void set_weapon_hit_block(bool value) { this->value = value ? (this->value | 0x200) : (this->value & ~0x200); }
        constexpr void set_hit_by_spike(bool value) { this->value = value ? (this->value | 0x400) : (this->value & ~0x400); }
        constexpr void set_water_splash_facing_right(bool value) { this->value = value ? (this->value | 0x800) : (this->value & ~0x800); }
        constexpr void set_force_left(bool value) { this->value = value ? (this->value | 0x1000) : (this->value & ~0x1000); }
        constexpr void set_force_up(bool value) { this->value = value ? (this->value | 0x2000) : (this->value & ~0x2000); }
        constexpr void set_force_right(bool value) { this->value = value ? (this->value | 0x4000) : (this->value & ~0x4000); }
        constexpr void set_force_down(bool value) { this->value = value ? (this->value | 0x8000) : (this->value & ~0x8000); }
        constexpr void set_hit_left_higher_half(bool value) { this->value = value ? (this->value | 0x10000) : (this->value & ~0x10000); }
        constexpr void set_hit_left_lower_half(bool value) { this->value = value ? (this->value | 0x20000) : (this->value & ~0x20000); }
        constexpr void set_hit_right_lower_half(bool value) { this->value = value ? (this->value | 0x40000) : (this->value & ~0x40000); }
        constexpr void set_hit_right_higher_half(bool value) { this->value = value ? (this->value | 0x80000) : (this->value & ~0x80000); }

        constexpr bool any_flag() const { return value != 0; }
        constexpr bool hit_anything() const { return (value & 0x2ff) != 0; }
    };

    struct Equipment
    {
        uint16_t value;

        constexpr Equipment() : value(0) {}
        constexpr Equipment(uint16_t value) : value(value) {}

        constexpr bool has_booster_0_8() const { return value & 0x01; }
        constexpr bool has_map() const { return value & 0x02; }
        constexpr bool has_arms_barrier() const { return value & 0x04; }
        constexpr bool has_turbocharge() const { return value & 0x08; }
        constexpr bool has_air_tank() const { return value & 0x10; }
        constexpr bool has_booster_2_0() const { return value & 0x20; }
        constexpr bool has_mimiga_mask() const { return value & 0x40; }
        constexpr bool has_whimsical_star() const { return value & 0x80; }
        constexpr bool has_nikumaru() const { return value & 0x100; }

        constexpr void set_booster_0_8(bool value) { this->value = value ? (this->value | 0x01) : (this->value & ~0x01); }
        constexpr void set_map(bool value) { this->value = value ? (this->value | 0x02) : (this->value & ~0x02); }
        constexpr void set_arms_barrier(bool value) { this->value = value ? (this->value | 0x04) : (this->value & ~0x04); }
        constexpr void set_turbocharge(bool value) { this->value = value ? (this->value | 0x08) : (this->value & ~0x08); }
        constexpr void set_air_tank(bool value) { this->value = value ? (this->value | 0x10) : (this->value & ~0x10); }
        constexpr void set_booster_2_0(bool value) { this->value = value ? (this->value | 0x20) : (this->value & ~0x20); }
        constexpr void set_mimiga_mask(bool value) { this->value = value ? (this->value | 0x40) : (this->value & ~0x40); }
        constexpr void set_whimsical_star(bool value) { this->value = value ? (this->value | 0x80) : (this->value & ~0x80); }
        constexpr void set_nikumaru(bool value) { this->value = value ? (this->value | 0x100) : (this->value & ~0x100); }
    };

    struct Condition
    {
        uint16_t value;

        constexpr Condition() : value(0) {}
        constexpr Condition(uint16_t value) : value(value) {}

        constexpr bool interacted() const { return value & 0x01; }
        constexpr bool hidden() const { return value & 0x02; }
        constexpr bool fallen() const { return value & 0x04; }
        constexpr bool explode_die() const { return value & 0x08; }
        constexpr bool damage_boss() const { return value & 0x10; }
        constexpr bool increase_acceleration() const { return value & 0x20; }
        constexpr bool cond_x40() const { return value & 0x40; }
        constexpr bool alive() const { return value & 0x80; }
        constexpr bool drs_novanish() const { return value & 0x4000; }
        constexpr bool drs_boss() const { return value & 0x8000; }

        constexpr void set_interacted(bool value) { this->value = value ? (this->value | 0x01) : (this->value & ~0x01); }
        constexpr void set_hidden(bool value) { this->value = value ? (this->value | 0x02) : (this->value & ~0x02); }
        constexpr void set_fallen(bool value) { this->value = value ? (this->value | 0x04) : (this->value & ~0x04); }
        constexpr void set_explode_die(bool value) { this->value = value ? (this->value | 0x08) : (this->value & ~0x08); }
        constexpr void set_damage_boss(bool value) { this->value = value ? (this->value | 0x10) : (this->value & ~0x10); }
        constexpr void set_increase_acceleration(bool value) { this->value = value ? (this->value | 0x20) : (this->value & ~0x20); }
        constexpr void set_cond_x40(bool value) { this->value = value ? (this->value | 0x40) : (this->value & ~0x40); }
        constexpr void set_alive(bool value) { this->value = value ? (this->value | 0x80) : (this->value & ~0x80); }
        constexpr void set_drs_novanish(bool value) { this->value = value ? (this->value | 0x4000) : (this->value & ~0x4000); }
        constexpr void set_drs_boss(bool value) { this->value = value ? (this->value | 0x8000) : (this->value & ~0x8000); }
    };

    struct ControlFlags
    {
        uint16_t value;
        constexpr ControlFlags() : value(0) {}
        constexpr ControlFlags(uint16_t value) : value(value) {}

        constexpr bool tick_world() const { return value & 0x01; }
        constexpr bool control_enabled() const { return value & 0x02; }
        constexpr bool interactions_disabled() const { return value & 0x04; }
        constexpr bool credits_running() const { return value & 0x08; }
        constexpr bool ok_button_disabled() const { return value & 0x10; }
        constexpr bool friendly_fire() const { return value & 0x4000; }

        constexpr void set_tick_world(bool value) { this->value = value ? (this->value | 0x01) : (this->value & ~0x01); }
        constexpr void set_control_enabled(bool value) { this->value = value ? (this->value | 0x02) : (this->value & ~0x02); }
        constexpr void set_interactions_disabled(bool value) { this->value = value ? (this->value | 0x04) : (this->value & ~0x04); }
        constexpr void set_credits_running(bool value) { this->value = value ? (this->value | 0x08) : (this->value & ~0x08); }
        constexpr void set_ok_button_disabled(bool value) { this->value = value ? (this->value | 0x10) : (this->value & ~0x10); }
        constexpr void set_friendly_fire(bool value) { this->value = value ? (this->value | 0x4000) : (this->value & ~0x4000); }
    };

    struct BulletFlag
    {
        uint8_t value;

        constexpr BulletFlag() : value(0) {}
        constexpr BulletFlag(uint8_t value) : value(value) {}

        // 0x01, nowhere in code?
        constexpr bool flag_x01() const { return value & 0x01; }
        // 0x02, nowhere in code?
        constexpr bool flag_x02() const { return value & 0x02; }
        // 0x04, if set, bullet will pass through blocks.
        constexpr bool no_collision_checks() const { return value & 0x04; }
        // 0x08, if set, bullet will bounce off walls.
        constexpr bool bounce_from_walls() const { return value & 0x08; }
        // 0x10, if set, bullet will not produce projectile dissipation effect when it hits a NPC or boss.
        constexpr bool no_proj_dissipation() const { return value & 0x10; }
        // 0x20, if set, performs checks in block collision check procedure. Kills the bullet if flag 0x40 isn't set.
        constexpr bool check_block_hit() const { return value & 0x20; }
        // 0x40, if set, bullet will destroy snack blocks on hit.
        constexpr bool can_destroy_snack() const { return value & 0x40; }
        // 0x80, nowhere in code?
        constexpr bool flag_x80() const { return value & 0x80; }

        constexpr void set_flag_x01(bool value) { this->value = value ? (this->value | 0x01) : (this->value & ~0x01); }
        constexpr void set_flag_x02(bool value) { this->value = value ? (this->value | 0x02) : (this->value & ~0x02); }
        constexpr void set_no_collision_checks(bool value) { this->value = value ? (this->value | 0x04) : (this->value & ~0x04); }
        constexpr void set_bounce_from_walls(bool value) { this->value = value ? (this->value | 0x08) : (this->value & ~0x08); }
        constexpr void set_no_proj_dissipation(bool value) { this->value = value ? (this->value | 0x10) : (this->value & ~0x10); }
        constexpr void set_check_block_hit(bool value) { this->value = value ? (this->value | 0x20) : (this->value & ~0x20); }
        constexpr void set_can_destroy_snack(bool value) { this->value = value ? (this->value | 0x40) : (this->value & ~0x40); }
        constexpr void set_flag_x80(bool value) { this->value = value ? (this->value | 0x80) : (this->value & ~0x80); }
    };

    class FadeDirection
    {
    public:
        enum Value : uint8_t
        {
            Left = 0,
            Up,
            Right,
            Down,
            Center,
        };

        FadeDirection() = default;
        constexpr FadeDirection(Value val) : value{val} {}

        constexpr operator Value() const { return value; }
        explicit operator bool() = delete;

        // C++20: Three-way comparison
        constexpr auto operator<=>(const FadeDirection&) const = default;
        constexpr bool operator==(const FadeDirection&) const = default;

        static constexpr std::optional<FadeDirection> from_int(int val)
        {
            switch (val) {
                case 0: return FadeDirection{Left};
                case 1: return FadeDirection{Up};
                case 2: return FadeDirection{Right};
                case 3: return FadeDirection{Down};
                case 4: return FadeDirection{Center};
                default: return std::nullopt;
            }
        }

        constexpr FadeDirection opposite() const
        {
            switch (value) {
                case Left: return Right;
                case Up: return Down;
                case Right: return Left;
                case Down: return Up;
                default: return Center;
            }
        }

    private:
        Value value;
    };

    class FadeState
    {
    public:
    };

    class Direction
    {
    public:
        enum Value : uint8_t
        {
            Left = 0,
            Up,
            Right,
            Bottom,
            FacingPlayer,
        };

        Direction() = default;
        constexpr Direction(Value val) : value{val} {}

        constexpr operator Value() const { return value; }
        explicit operator bool() = delete;

        // C++20: Use three-way comparison instead of separate == and !=
        constexpr auto operator<=>(const Direction& other) const = default;
        constexpr bool operator==(const Direction& other) const = default;

        constexpr bool operator==(Value other) const { return value == other; }
        constexpr bool operator!=(Value other) const { return value != other; }

        static constexpr std::optional<Direction> from_int(int val)
        {
            switch (val) {
                case 0: return Direction{Left};
                case 1: return Direction{Up};
                case 2: return Direction{Right};
                case 3: return Direction{Bottom};
                case 4: return Direction{FacingPlayer};
                default: return std::nullopt;
            }
        }

        constexpr Direction opposite() const
        {
            switch (value) {
                case Left: return Right;
                case Up: return Bottom;
                case Right: return Left;
                case Bottom: return Up;
                default: return FacingPlayer;
            }
        }

        constexpr int vector_x() const
        {
            switch (value) {
                case Left: return -1;
                case Right: return 1;
                default: return 0;
            }
        }

        constexpr int vector_y() const
        {
            switch (value) {
                case Up: return -1;
                case Bottom: return 1;
                default: return 0;
            }
        }

    private:
        Value value;
    };

    // C++20: Use concepts to constrain template parameter for arithmetic types
    template <typename T>
    requires std::is_arithmetic_v<T>
    struct Rect
    {
        T left;
        T top;
        T right;
        T bottom;

        constexpr Rect() = default;
        
        // Traditional constructor (designated initializers don't work with constructors)
        constexpr Rect(T left, T top, T right, T bottom) : left{left}, top{top}, right{right}, bottom{bottom} {}

        constexpr static Rect new_size(T x, T y, T width, T height)
        {
            return Rect{x, y, x + width, y + height};
        }

        constexpr bool has_point(T x, T y) const
        {
            return left <= x && x <= right && top <= y && y <= bottom;
        }

        constexpr T width() const
        {
            if constexpr (std::is_floating_point_v<T>) {
                return std::abs(right - left);
            } else {
                return left > right ? left - right : right - left;
            }
        }

        constexpr T height() const
        {
            if constexpr (std::is_floating_point_v<T>) {
                return std::abs(bottom - top);
            } else {
                return top > bottom ? top - bottom : bottom - top;
            }
        }

        // C++20: Three-way comparison operator
        constexpr auto operator<=>(const Rect&) const = default;
        constexpr bool operator==(const Rect&) const = default;
    };

    inline float fix9_scale(int val)
    {
        return (float)val * doukutsu_rs::texture_set::G_MAG / 512.0f;
    }

    inline double lerp_f64(double v1, double v2, double t)
    {
        return v1 * (1.0 - t) + v2 * t;
    }

    float interpolate_fix9_scale(int old_val, int val, float frame_delta);

    uint64_t get_timestamp();

    struct Color
    {
        float r, g, b, a;

        constexpr Color() : r{0.0f}, g{0.0f}, b{0.0f}, a{0.0f} {}
        constexpr Color(float r, float g, float b, float a) : r{r}, g{g}, b{b}, a{a} {}

        constexpr static Color from_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a)
        {
            return Color{r / 255.0f, g / 255.0f, b / 255.0f, a / 255.0f};
        }

        constexpr static Color from_rgb(uint8_t r, uint8_t g, uint8_t b)
        {
            return Color{r / 255.0f, g / 255.0f, b / 255.0f, 1.0f};
        }

        constexpr static Color from_rgba_u32(uint32_t c)
        {
            return Color::from_rgba(
                static_cast<uint8_t>(c >> 24),
                static_cast<uint8_t>(c >> 16),
                static_cast<uint8_t>(c >> 8),
                static_cast<uint8_t>(c));
        }

        constexpr static Color from_rgb_u32(uint32_t c)
        {
            return Color::from_rgb(
                static_cast<uint8_t>(c >> 16),
                static_cast<uint8_t>(c >> 8),
                static_cast<uint8_t>(c));
        }

        constexpr std::tuple<uint8_t, uint8_t, uint8_t, uint8_t> to_rgba() const
        {
            return std::make_tuple(
                static_cast<uint8_t>(r * 255.0f),
                static_cast<uint8_t>(g * 255.0f),
                static_cast<uint8_t>(b * 255.0f),
                static_cast<uint8_t>(a * 255.0f));
        }

        constexpr std::tuple<uint8_t, uint8_t, uint8_t> to_rgb() const
        {
            return std::make_tuple(
                static_cast<uint8_t>(r * 255.0f),
                static_cast<uint8_t>(g * 255.0f),
                static_cast<uint8_t>(b * 255.0f));
        }

        constexpr uint32_t to_rgba_u32() const
        {
            const auto [r_val, g_val, b_val, a_val] = to_rgba();
            return static_cast<uint32_t>(a_val) << 24 | 
                   static_cast<uint32_t>(r_val) << 16 | 
                   static_cast<uint32_t>(g_val) << 8 | 
                   static_cast<uint32_t>(b_val);
        }

        constexpr uint32_t to_rgb_u32() const
        {
            const auto [r_val, g_val, b_val] = to_rgb();
            return static_cast<uint32_t>(r_val) << 16 | 
                   static_cast<uint32_t>(g_val) << 8 | 
                   static_cast<uint32_t>(b_val);
        }

        // C++20: Three-way comparison
        constexpr auto operator<=>(const Color&) const = default;
        constexpr bool operator==(const Color&) const = default;
    };

    [[noreturn]]
    inline void unreachable() {
        __builtin_unreachable();
    }
};