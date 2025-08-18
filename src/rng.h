#pragma once

#include <cstdint>
#include <tuple>

namespace doukutsu::rng
{
    class RNG
    {
    public:
        virtual ~RNG() = default;  // C++20: Always good practice for virtual classes
        virtual int next() = 0;

        int range(int start, int end);
    };

    class XorShift : public RNG
    {
    private:
        uint64_t state;

    public:
        explicit XorShift(int seed) : state{static_cast<uint64_t>(seed)} {}

        int next() override;
        uint64_t next_u64();

        // C++20: Use [[nodiscard]] for functions that should not ignore return values
        [[nodiscard]] uint32_t next_u32()
        {
            return static_cast<uint32_t>(next_u64() >> 32);
        }

        [[nodiscard]] constexpr uint64_t dump_state() const noexcept
        {
            return state;
        }

        constexpr void load_state(uint64_t saved_state) noexcept
        {
            state = saved_state;
        }
    };

    class Xoroshiro32PlusPlus : public RNG
    {
    private:
        std::pair<uint16_t, uint16_t> state;

    public:
        explicit Xoroshiro32PlusPlus(uint32_t seed) 
            : state{static_cast<uint16_t>(seed & 0xffff), static_cast<uint16_t>((seed >> 16) & 0xffff)} {};

        int next() override;

        [[nodiscard]] uint16_t next_u16();

        [[nodiscard]] constexpr uint32_t dump_state() const noexcept
        {
            return static_cast<uint32_t>(state.first) | (static_cast<uint32_t>(state.second) << 16);
        }

        constexpr void load_state(uint32_t new_state) noexcept
        {
            state = {static_cast<uint16_t>(new_state & 0xffff), 
                     static_cast<uint16_t>((new_state >> 16) & 0xffff)};
        }
    };
};
