#include "../../shared_game_state.h"
#include "../../npc.h"

using namespace doukutsu_rs;
using namespace doukutsu_rs::common;
using namespace doukutsu_rs::shared_game_state;
using namespace doukutsu_rs::npc;

void NPC::tick_n254_helicopter(SharedGameState &state, NPCList &npc_list)
{
    switch (this->action_num)
    {
    case 0:
    {
        this->action_num = 1;

        // blades
        auto npc = NPC::create(255, state.npc_table);
        npc.cond.set_alive(true);
        npc.x = this->x + 0x2400;
        npc.y = this->y - 0x7200;
        npc.parent_id = this->id;

        npc_list.spawn(0x100, npc);

        npc.x = this->x - 0x4000;
        npc.y = this->y - 0x6800;
        npc.direction = Direction::Right;

        npc_list.spawn(0x100, npc);
    }
    case 20:
    {
        this->action_num = 21;
        this->action_counter = 0;
        this->action_counter2 = 60;
    }
    case 30:
    {
        this->action_num = 21;

        // momorin
        auto npc = NPC::create(223, state.npc_table);
        npc.cond.set_alive(true);
        npc.x = this->x - 0x1600;
        npc.y = this->y - 0x1c00;

        npc_list.spawn(0x100, npc);
    }
    case 40:
    {
        this->action_num = 21;

        // momorin
        auto npc = NPC::create(223, state.npc_table);
        npc.cond.set_alive(true);
        npc.x = this->x - 0x1200;
        npc.y = this->y - 0x1c00;

        npc_list.spawn(0x100, npc);

        // santa
        auto npc = NPC::create(40, state.npc_table);
        npc.cond.set_alive(true);
        npc.x = this->x - 0x2c00;
        npc.y = this->y - 0x1c00;

        npc_list.spawn(0x100, npc);

        // chaco
        auto npc = NPC::create(223, state.npc_table);
        npc.cond.set_alive(true);
        npc.x = this->x - 0x4600;
        npc.y = this->y - 0x1c00;

        npc_list.spawn(0x100, npc);
    }
    default:
        break;
    }

    this->anim_rect = state.constants.npc.n254_helicopter[this->direction == Direction::Left ? 0 : 1];
}

void NPC::tick_n255_helicopter_blades(SharedGameState &state, NPCList &npc_list)
{
    switch (this->action_num)
    {
    case 0:
    {
        this->action_num = 1;

        if (this->direction == Direction::Left)
        {
            this->display_bounds.left = 0x7000;
            this->display_bounds.right = 0x7000;
        }
        else
        {
            this->display_bounds.left = 0x5000;
            this->display_bounds.right = 0x5000;
        }
    }
    case 10:
    {
        this->action_num = 11;

        this->anim_num += 1;
        if (this->anim_num > 3)
        {
            this->anim_num = 0;
        }
    }
    default:
        break;
    }

    this->anim_rect = state.constants.npc.n255_helicopter_blades[this->anim_num + (this->direction == Direction::Left ? 0 : 4)];
}

void NPC::tick_n260_shovel_brigade_caged(SharedGameState &state, NPCList &npc_list)
{
    switch (this->action_num)
    {
    case 0:
    case 1:
    {
        if (this->action_num == 0)
        {
            this->x += 0x200;
            this->y -= 0x400;
            this->action_num = 1;
            this->anim_num = 0;
            this->anim_counter = 0;
        }

        if (this->rng.range(0, 160) == 1)
        {
            this->action_num = 2;
            this->action_counter = 0;
            this->anim_num = 1;
        }
    }
    case 2:
    {
        this->action_counter += 1;
        if (this->action_counter > 12)
        {
            this->action_num = 1;
            this->anim_num = 0;
        }
    }
    case 10:
    {
        this->action_num = 11;
        this->anim_num = 2;

        // create heart
        auto npc = NPC::create(87, state.npc_table);
        npc.cond.set_alive(true);
        npc.x = this->x;
        npc.y = this->y - 0x2000;

        npc_list.spawn(0x100, npc);
    }
    default:
        break;
    }

    this->anim_rect = state.constants.npc.n260_shovel_brigade_caged[this->anim_num + (this->direction == Direction::Left ? 0 : 3)];
}

void NPC::tick_n261_chie_caged(SharedGameState &state, NPCList &npc_list, Players &players)
{
    switch (this->action_num)
    {
    case 0:
    case 1:
    {
        if (this->action_num == 0)
        {
            this->x -= 0x200;
            this->y -= 0x400;
            this->action_num = 1;
            this->anim_num = 0;
            this->anim_counter = 0;
        }

        if (this->rng.range(0, 160) == 1)
        {
            this->action_num = 2;
            this->action_counter = 0;
            this->anim_num = 1;
        }
    }
    case 2:
    {
        this->action_counter += 1;
        if (this->action_counter > 12)
        {
            this->action_num = 1;
            this->anim_num = 0;
        }
    }
    default:
        break;
    }

    auto player = this->get_closest_player_ref(players);
    this->face_player(player);

    this->anim_rect = state.constants.npc.n261_chie_caged[this->anim_num + (this->direction == Direction::Left ? 0 : 3)];
}

void NPC::tick_n262_chaco_caged(SharedGameState &state, NPCList &npc_list, Players &players)
{
    switch (this->action_num)
    {
    case 0:
    case 1:
    {
        if (this->action_num == 0)
        {
            this->x -= 0x200;
            this->y -= 0x400;
            this->action_num = 1;
            this->anim_num = 0;
            this->anim_counter = 0;
        }

        if (this->rng.range(0, 160) == 1)
        {
            this->action_num = 2;
            this->action_counter = 0;
            this->anim_num = 1;
        }
    }
    case 2:
    {
        this->action_counter += 1;
        if (this->action_counter > 12)
        {
            this->action_num = 1;
            this->anim_num = 0;
        }
    }
    default:
        break;
    }

    auto player = this->get_closest_player_ref(players);
    this->face_player(player);

    this->anim_rect = state.constants.npc.n262_chaco_caged[this->anim_num + (this->direction == Direction::Left ? 0 : 3)];
}