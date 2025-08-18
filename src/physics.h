#pragma once

#include "common.h"
#include "map.h"
#include <array>

namespace doukutsu {

// Forward declarations - stubs for now
class SharedGameState {
public:
    int32_t water_level = 0x3200000; // Default water level
    // Other members will be added as needed
};

namespace physics {

/// Collision check offsets - based on Rust physics.rs OFFSETS
extern const std::array<std::pair<int32_t, int32_t>, 64> OFFSETS;

/// Physical entity interface for collision detection
class PhysicalEntity {
public:
    virtual ~PhysicalEntity() = default;

    // Position and velocity getters
    [[nodiscard]] virtual int32_t x() const = 0;
    [[nodiscard]] virtual int32_t y() const = 0;
    [[nodiscard]] virtual int32_t vel_x() const = 0;
    [[nodiscard]] virtual int32_t vel_y() const = 0;

    // Collision properties
    [[nodiscard]] virtual size_t hit_rect_size() const = 0;
    [[nodiscard]] virtual int32_t offset_x() const { return 0; }
    [[nodiscard]] virtual int32_t offset_y() const { return 0; }

    [[nodiscard]] virtual const doukutsu::common::Rect<uint32_t>& hit_bounds() const = 0;
    [[nodiscard]] virtual const doukutsu::common::Rect<uint32_t>& display_bounds() const = 0;

    // Position and velocity setters
    virtual void set_x(int32_t x) = 0;
    virtual void set_y(int32_t y) = 0;
    virtual void set_vel_x(int32_t vel_x) = 0;
    virtual void set_vel_y(int32_t vel_y) = 0;

    // Entity state
    virtual doukutsu::common::Condition& cond() = 0;
    virtual doukutsu::common::Flag& flags() = 0;

    // Entity properties
    [[nodiscard]] virtual doukutsu::common::Direction direction() const = 0;
    [[nodiscard]] virtual bool is_player() const = 0;
    [[nodiscard]] virtual bool ignore_tile_44() const { return true; }
    [[nodiscard]] virtual bool player_left_pressed() const { return false; }
    [[nodiscard]] virtual bool player_right_pressed() const { return false; }

    // Collision testing methods (stubs for now)
    virtual void test_block_hit(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_platform_hit(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_water(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_spike(SharedGameState& state, int32_t x, int32_t y, bool water) {}
    virtual void test_hit_force(SharedGameState& state, int32_t x, int32_t y, 
                               doukutsu::common::Direction direction, bool water) {}

    // Slope collision methods (stubs for now)
    virtual void test_hit_upper_left_slope_high(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_upper_left_slope_low(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_upper_right_slope_low(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_upper_right_slope_high(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_lower_left_slope_high(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_lower_left_slope_low(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_lower_right_slope_low(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_lower_right_slope_high(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_upper_left_slope(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_upper_right_slope(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_lower_left_slope(SharedGameState& state, int32_t x, int32_t y) {}
    virtual void test_hit_lower_right_slope(SharedGameState& state, int32_t x, int32_t y) {}

    /// Main collision processing method
    virtual void tick_map_collisions(SharedGameState& state, const Map& map);
};

} // namespace physics
} // namespace doukutsu