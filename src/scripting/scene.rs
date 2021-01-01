use lua_ffi::{c_int, LuaObject, State};
use lua_ffi::ffi::luaL_Reg;

use crate::inventory::Inventory;
use crate::player::Player;
use crate::scene::game_scene::GameScene;
use crate::scripting::LuaScriptingState;
use crate::scripting::player::LuaPlayer;

pub struct LuaGameScene {
    valid_reference: bool,
    ptr: *mut GameScene,
}

impl LuaGameScene {
    unsafe fn lua_get_tick(&self, state: &mut State) -> c_int {
        state.push((*self.ptr).tick as u32);

        1
    }

    unsafe fn lua_get_player(&self, state: &mut State) -> c_int {
        if let Some(index) = state.to_int(2) {
            let (player_ref, inv_ref) = match index {
                0 => (&mut (*self.ptr).player1, &mut (*self.ptr).inventory_player1),
                1 => (&mut (*self.ptr).player2, &mut (*self.ptr).inventory_player2),
                _ => {
                    state.error("Player index out of range!");
                    return 0;
                }
            };

            state.push(LuaPlayer::new(player_ref as *mut Player, inv_ref as *mut Inventory));
            1
        } else {
            state.error("Player index must be a number.");
            0
        }
    }

    pub(crate) fn new(ptr: *mut GameScene) -> LuaGameScene {
        LuaGameScene {
            valid_reference: true,
            ptr,
        }
    }
}

impl Drop for LuaGameScene {
    fn drop(&mut self) {
        self.valid_reference = false;
    }
}

impl LuaObject for LuaGameScene {
    fn name() -> *const i8 {
        c_str!("GameScene")
    }

    fn lua_fns() -> Vec<luaL_Reg> {
        vec![
            lua_method!("tick", LuaGameScene, LuaGameScene::lua_get_tick),
            lua_method!("player", LuaGameScene, LuaGameScene::lua_get_player),
        ]
    }
}

impl LuaScriptingState {
    pub fn scene_tick(&mut self, game_scene: &mut GameScene) {
        self.game_scene = game_scene as *mut GameScene;

        if let Some(state) = self.state.as_mut() {
            let val = LuaGameScene::new(self.game_scene);

            state.get_global("doukutsu");
            state.get_field(-1, "_handlers");
            state.get_field(-1, "tick");

            state.push(val);
            if let Err((_, err)) = state.pcall(1, 0, 0) {
                println!("scene_tick error: {}", err);
            }

            state.pop(2);
        }
    }
}
