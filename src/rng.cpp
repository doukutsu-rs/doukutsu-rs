#include "rng.h"

using namespace doukutsu_rs::rng;

int RNG::range(int start, int end)
{
    return start + ((next() & 0x7fffffff) % (end - start + 1));
}

int XorShift::next()
{
    return (int)next_u32();
}

uint64_t XorShift::next_u64()
{
    uint64_t state = state;

    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;

    state = state;

    return state * 0x2545F4914F6CDD1DLL;
}

int Xoroshiro32PlusPlus::next()
{
    return (int)((next_u16() << 16) | next_u16());
}

uint16_t Xoroshiro32PlusPlus::next_u16()
{
    uint16_t result = state.first + state.second;
    result = result << 9 | result >> 7;
    result += state.first;

    state.second ^= state.first;
    state.first = (state.first << 13 | state.first >> 3) ^ state.second ^ (state.second << 5);
    state.second = state.second << 10 | state.second >> 6;

    return result;
}