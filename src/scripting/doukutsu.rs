use std::io::Read;

use lua_ffi::ffi::luaL_Reg;
use lua_ffi::{c_int, LuaObject, State};

use crate::common::{Direction, Rect};
use crate::framework::filesystem;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::scene::game_scene::GameScene;
use crate::scripting::{check_status, LuaScriptingState, DRS_RUNTIME_GLOBAL};

pub struct Doukutsu {
    pub ptr: *mut LuaScriptingState,
}

#[allow(unused)]
impl Doukutsu {
    pub fn new(ptr: *mut LuaScriptingState) -> Doukutsu {
        Doukutsu { ptr }
    }

    unsafe fn lua_play_sfx(&self, state: &mut State) -> c_int {
        if let Some(index) = state.to_int(2) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            game_state.sound_manager.play_sfx(index as u8);
        }

        0
    }

    unsafe fn lua_play_sfx_loop(&self, state: &mut State) -> c_int {
        if let Some(index) = state.to_int(2) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            game_state.sound_manager.loop_sfx(index as u8);
        }

        0
    }

    unsafe fn lua_play_song(&self, state: &mut State) -> c_int {
        if let Some(index) = state.to_int(2) {
            let game_state = &mut (*(*self.ptr).state_ptr);
            let ctx = &mut (*(*self.ptr).ctx_ptr);

            let _ =
                game_state.sound_manager.play_song(index as usize, &game_state.constants, &game_state.settings, ctx);
        }

        0
    }

    unsafe fn lua_set_setting(&self, state: &mut State) -> c_int {
        if let Some(index) = state.to_int(2) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            state.push(game_state.get_flag(index.max(0) as usize));
        } else {
            state.push_nil();
        }

        1
    }

    unsafe fn lua_get_flag(&self, state: &mut State) -> c_int {
        if let Some(index) = state.to_int(2) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            state.push(game_state.get_flag(index.max(0) as usize));
        } else {
            state.push_nil();
        }

        1
    }

    unsafe fn lua_set_flag(&self, state: &mut State) -> c_int {
        let flag_id = state.to_int(2);
        let flag_val = state.to_bool(3);

        if let (Some(flag_id), Some(flag_val)) = (flag_id, flag_val) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            game_state.set_flag(flag_id.max(0) as usize, flag_val);
        }

        0
    }

    unsafe fn lua_get_skip_flag(&self, state: &mut State) -> c_int {
        if let Some(index) = state.to_int(2) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            state.push(game_state.get_skip_flag(index.max(0) as usize));
        } else {
            state.push_nil();
        }

        1
    }

    unsafe fn lua_set_skip_flag(&self, state: &mut State) -> c_int {
        let flag_id = state.to_int(2);
        let flag_val = state.to_bool(3);

        if let (Some(flag_id), Some(flag_val)) = (flag_id, flag_val) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            game_state.set_skip_flag(flag_id.max(0) as usize, flag_val);
        }

        0
    }

    unsafe fn lua_set_engine_constant(&self, state: &mut State) -> c_int {
        if let Some(constant_id) = state.to_int(2) {
            let game_state = &mut (*(*self.ptr).state_ptr);

            match constant_id {
                0x1000 => {
                    // intro event
                    if let Some(intro_event) = state.to_int(3) {
                        game_state.constants.game.intro_event = intro_event as u16;
                    }
                }
                0x1001 => {
                    // intro stage
                    if let Some(intro_stage) = state.to_int(3) {
                        game_state.constants.game.intro_stage = intro_stage as u16;
                    }
                }
                0x1002 => {
                    // intro pos
                    if let (Some(intro_x), Some(intro_y)) = (state.to_int(3), state.to_int(4)) {
                        game_state.constants.game.intro_player_pos = (intro_x as i16, intro_y as i16);
                    }
                }
                0x1003 => {
                    // ng event
                    if let Some(ng_event) = state.to_int(3) {
                        game_state.constants.game.new_game_event = ng_event as u16;
                    }
                }
                0x1004 => {
                    // ng stage
                    if let Some(ng_stage) = state.to_int(3) {
                        game_state.constants.game.new_game_stage = ng_stage as u16;
                    }
                }
                0x1005 => {
                    // ng pos
                    if let (Some(ng_x), Some(ng_y)) = (state.to_int(3), state.to_int(4)) {
                        game_state.constants.game.new_game_player_pos = (ng_x as i16, ng_y as i16);
                    }
                }
                _ => {}
            }
        }

        0
    }

    unsafe fn lua_npc_command(&self, state: &mut State) -> c_int {
        if (*self.ptr).game_scene.is_null() {
            state.push_nil();
            return 1;
        }

        if let (Some(npc_id), Some(param_type)) = (state.to_int(2), state.to_int(3)) {
            let game_scene = &mut *(*self.ptr).game_scene;

            let npc = match game_scene.npc_list.get_npc(npc_id as usize) {
                Some(npc) => npc,
                None => {
                    state.push_nil();
                    return 1;
                }
            };

            match param_type {
                0x0e => state.push(npc.cond.0 as i32),
                0x0f => state.push(npc.flags.0),
                0x10 => state.push(npc.x as f32 / 512.0),
                0x11 => state.push(npc.y as f32 / 512.0),
                0x12 => state.push(npc.vel_x as f32 / 512.0),
                0x13 => state.push(npc.vel_y as f32 / 512.0),
                0x14 => state.push(npc.vel_x2 as f32 / 512.0),
                0x15 => state.push(npc.vel_y2 as f32 / 512.0),
                0x16 => state.push(npc.action_num as i32),
                0x17 => state.push(npc.anim_num as i32),
                0x18 => state.push(npc.action_counter as i32),
                0x19 => state.push(npc.action_counter2 as i32),
                0x1a => state.push(npc.action_counter3 as i32),
                0x1b => state.push(npc.anim_counter as i32),
                0x1c => state.push(npc.parent_id as i32),
                0x1d => state.push(npc.npc_type as i32),
                0x1e => state.push(npc.life as i32),
                0x1f => state.push(npc.flag_num as i32),
                0x20 => state.push(npc.event_num as i32),
                0x21 => state.push(npc.direction as i32),
                0x22 => state.push(npc.tsc_direction as i32),
                0x10e => {
                    if let Some(v) = state.to_uint(4) {
                        npc.cond.0 = v as u16;
                    }
                }
                0x10f => {
                    if let Some(v) = state.to_uint(4) {
                        npc.flags.0 = v;
                    }
                }
                0x110 => {
                    if let Some(v) = state.to_float(4) {
                        // set x
                        npc.x = (v * 512.0) as i32;
                    }
                }
                0x111 => {
                    if let Some(v) = state.to_float(4) {
                        // set y
                        npc.y = (v * 512.0) as i32;
                    }
                }
                0x112 => {
                    if let Some(v) = state.to_float(4) {
                        // set vel x
                        npc.vel_x = (v * 512.0) as i32;
                    }
                }
                0x113 => {
                    if let Some(v) = state.to_float(4) {
                        // set vel y
                        npc.vel_y = (v * 512.0) as i32;
                    }
                }
                0x114 => {
                    if let Some(v) = state.to_float(4) {
                        // set vel x 2
                        npc.vel_x2 = (v * 512.0) as i32;
                    }
                }
                0x115 => {
                    if let Some(v) = state.to_float(4) {
                        // set vel y 2
                        npc.vel_y2 = (v * 512.0) as i32;
                    }
                }
                0x116 => {
                    if let Some(v) = state.to_int(4) {
                        npc.action_num = v as u16;
                    }
                }
                0x117 => {
                    if let Some(v) = state.to_int(4) {
                        npc.anim_num = v as u16;
                    }
                }
                0x118 => {
                    if let Some(v) = state.to_int(4) {
                        npc.action_counter = v as u16;
                    }
                }
                0x119 => {
                    if let Some(v) = state.to_int(4) {
                        npc.action_counter2 = v as u16;
                    }
                }
                0x11a => {
                    if let Some(v) = state.to_int(4) {
                        npc.action_counter3 = v as u16;
                    }
                }
                0x11b => {
                    if let Some(v) = state.to_int(4) {
                        npc.anim_counter = v as u16;
                    }
                }
                0x11c => {
                    if let Some(v) = state.to_int(4) {
                        npc.parent_id = v as u16;
                    }
                }
                0x11d => {
                    if let Some(v) = state.to_int(4) {
                        npc.npc_type = v as u16;
                    }
                }
                0x11e => {
                    if let Some(v) = state.to_int(4) {
                        npc.life = v as u16;
                    }
                }
                0x11f => {
                    if let Some(v) = state.to_int(4) {
                        npc.flag_num = v as u16;
                    }
                }
                0x120 => {
                    if let Some(v) = state.to_int(4) {
                        npc.event_num = v as u16;
                    }
                }
                0x121 | 0x122 => {
                    if let Some(v) = state.to_int(4) {
                        npc.direction = Direction::from_int_facing(v as _).unwrap_or(Direction::Left);
                        npc.tsc_direction = v as _;
                    }
                }
                0x200 => {
                    // get player idx
                    let index = npc.get_closest_player_idx_mut(&[&mut game_scene.player1, &mut game_scene.player2]);
                    state.push(index as i32);
                }
                0x201 => {
                    // random
                    if let (Some(min), Some(max)) = (state.to_int(4), state.to_int(5)) {
                        if max < min {
                            state.error("max < min");
                        } else {
                            state.push(npc.rng.range(min..max));
                        }
                    } else {
                        state.error("Invalid parameters supplied.");
                    }
                }
                0x202 => {
                    // get anim rect
                    state.push(npc.anim_rect.left as i32);
                    state.push(npc.anim_rect.top as i32);
                    state.push(npc.anim_rect.right as i32);
                    state.push(npc.anim_rect.bottom as i32);
                }
                0x203 => {
                    // set anim rect
                    if let (Some(l), Some(t), Some(r), Some(b)) =
                        (state.to_int(4), state.to_int(5), state.to_int(6), state.to_int(7))
                    {
                        npc.anim_rect = Rect { left: l as u16, top: t as u16, right: r as u16, bottom: b as u16 };
                    } else {
                        state.error("Invalid parameters supplied.");
                    }
                }
                _ => state.push_nil(),
            }
        } else {
            state.push_nil()
        }

        1
    }

    unsafe fn lua_player_command(&self, state: &mut State) -> c_int {
        if (*self.ptr).game_scene.is_null() {
            state.push_nil();
            return 1;
        }

        if let (Some(player_id), Some(param_type)) = (state.to_int(2), state.to_int(3)) {
            let game_scene = &mut *(*self.ptr).game_scene;

            let player = match player_id {
                0 => &mut game_scene.player1,
                1 => &mut game_scene.player2,
                _ => {
                    state.push_nil();
                    return 1;
                }
            };

            match param_type {
                0x0e => state.push(player.cond.0 as u32),
                0x0f => state.push(player.flags.0),
                0x10 => state.push(player.x as f32 / 512.0),
                0x11 => state.push(player.y as f32 / 512.0),
                0x12 => state.push(player.vel_x as f32 / 512.0),
                0x13 => state.push(player.vel_y as f32 / 512.0),
                0x110 => {
                    if let Some(v) = state.to_float(4) {
                        // set x
                        player.x = (v * 512.0) as i32;
                    }
                }
                0x111 => {
                    if let Some(v) = state.to_float(4) {
                        // set y
                        player.y = (v * 512.0) as i32;
                    }
                }
                0x112 => {
                    if let Some(v) = state.to_float(4) {
                        // set vel x
                        player.vel_x = (v * 512.0) as i32;
                    }
                }
                0x113 => {
                    if let Some(v) = state.to_float(4) {
                        // set vel y
                        player.vel_y = (v * 512.0) as i32;
                    }
                }
                0x200 => {
                    if let Some(points) = state.to_int(4) {
                        player.damage(points, &mut (*(*self.ptr).state_ptr), &game_scene.npc_list);
                    }

                    state.push_nil();
                }
                _ => state.push_nil(),
            }
        } else {
            state.push_nil()
        }

        1
    }

    unsafe fn lua_load_script(&mut self, state: &mut State) -> c_int {
        let lua_state = &mut (*self.ptr);

        if let Some(name) = state.to_str(2) {
            let name = name.to_string();

            let ctx = &mut (*(*self.ptr).ctx_ptr);

            let path = format!("/Scripts/{}.lua", name);
            let lua_vfs_path = format!("@/Scripts/{}.lua", name);

            fn raise_error(name: &str, state: &mut State, err: &str) {
                let error_msg = format!("module '{}' not found: {}", name, err.to_string());
                state.error(&error_msg);
            }

            match filesystem::open(ctx, &path) {
                Ok(mut file) => {
                    let mut buf = Vec::new();
                    if let Err(res) = file.read_to_end(&mut buf) {
                        raise_error(&name, state, &res.to_string());
                        return 0;
                    }

                    let res = state.load_buffer(&buf, &lua_vfs_path);
                    if let Err(err) = check_status(res, state) {
                        raise_error(&name, state, &err.to_string());
                        return 0;
                    }

                    return match state.pcall(0, 1, 0) {
                        Ok(_) => 1,
                        Err((_, err)) => {
                            raise_error(&name, state, &err);
                            0
                        }
                    };
                }
                Err(err) => {
                    raise_error(&name, state, &err.to_string());
                }
            }
        }

        0
    }
}

impl LuaObject for Doukutsu {
    fn name() -> *const i8 {
        c_str!("doukutsu-rs-internal")
    }

    fn lua_fns() -> Vec<luaL_Reg> {
        vec![
            lua_method!("playSfx", Doukutsu, Doukutsu::lua_play_sfx),
            lua_method!("playSong", Doukutsu, Doukutsu::lua_play_song),
            lua_method!("getFlag", Doukutsu, Doukutsu::lua_get_flag),
            lua_method!("setFlag", Doukutsu, Doukutsu::lua_set_flag),
            lua_method!("getSkipFlag", Doukutsu, Doukutsu::lua_get_skip_flag),
            lua_method!("setSkipFlag", Doukutsu, Doukutsu::lua_set_skip_flag),
            lua_method!("setEngineConstant", Doukutsu, Doukutsu::lua_set_engine_constant),
            lua_method!("playerCommand", Doukutsu, Doukutsu::lua_player_command),
            lua_method!("npcCommand", Doukutsu, Doukutsu::lua_npc_command),
            lua_method!("loadScript", Doukutsu, Doukutsu::lua_load_script),
        ]
    }
}

impl LuaScriptingState {
    pub fn try_run_npc_hook(&mut self, npc_id: u16, npc_type: u16) -> bool {
        let mut result = false;

        if let Some(state) = self.state.as_mut() {
            state.get_global(DRS_RUNTIME_GLOBAL);
            state.get_field(-1, "_tryNPCHook");

            state.push(npc_id as i32);
            state.push(npc_type as i32);

            if let Err((_, err)) = state.pcall(2, 1, 0) {
                log::error!("npc_hook error: {}", err);
            }

            if let Some(val) = state.to_bool(-1) {
                result = val;
            }

            state.pop(2);
        }

        result
    }
}
