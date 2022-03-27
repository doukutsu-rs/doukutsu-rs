#include "npc.h"

using namespace doukutsu_rs::npc;

NPCList::NPCList() : max_npc(0), npcs({})
{
}

NPCList::~NPCList()
{
}

bool NPCList::spawn(uint16_t min_id, NPC npc)
{
    auto npc_len = npcs.size();

    if (min_id >= npc_len)
    {
        return false;
    }

    for (auto id = min_id; id < npc_len; id++)
    {
        auto npc_ref = &npcs[id];

        if (!npc_ref->cond.alive())
        {
            npc.id = id;

            if (npc.tsc_direction == 0)
            {
                npc.tsc_direction = (uint16_t)npc.direction;
            }

            npc.init_rng();

            *npc_ref = npc;

            if (max_npc <= id)
            {
                max_npc = id + 1;
            }

            return true;
        }
    }

    return false;
}

bool NPCList::spawn_at_slot(uint16_t id, NPC npc)
{
    auto npc_len = npcs.size();

    if (id >= npc_len)
    {
        return false;
    }

    npc.id = id;

    if (npc.tsc_direction == 0)
    {
        npc.tsc_direction = (uint16_t)npc.direction;
    }

    npc.init_rng();

    npcs[id] = npc;

    if (max_npc <= id)
    {
        max_npc = id + 1;
    }

    return true;
}

std::optional<NPC &> NPCList::get_npc(size_t id)
{
    if (id >= npcs.size())
    {
        return std::nullopt;
    }

    auto &npc = npcs[id];
    return {npc};
}