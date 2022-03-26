#pragma once

#include <cstdint>
#include <tuple>

namespace doukutsu_rs::rng
{
    class RNG
    {
    public:
        virtual int next() = 0;

        int range(int start, int end);
    };

    class XorShift : public RNG
    {
    private:
        uint64_t state;

    public:
        XorShift(int seed) : state(seed){};

        virtual int next() override;
        uint64_t next_u64();

        inline uint32_t next_u32()
        {
            return next_u64() >> 32;
        }

        uint64_t dump_state() const
        {
            return state;
        }

        void load_state(uint64_t saved_state)
        {
            state = saved_state;
        }
    };

    class Xoroshiro32PlusPlus : public RNG
    {
    private:
        std::pair<uint16_t, uint16_t> state;

    public:
        Xoroshiro32PlusPlus(uint32_t seed) : state({seed & 0xffff, (seed >> 16) & 0xffff}){};

        virtual int next() override;

        uint16_t next_u16();

        uint32_t dump_state() const
        {
            return state.first | (state.second << 16);
        }

        void load_state(uint32_t state)
        {
            this->state = {state & 0xffff, (state >> 16) & 0xffff};
        }
    };
};
