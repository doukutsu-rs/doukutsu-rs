use lua_ffi::{c_int, LuaObject, State};
use lua_ffi::c_str;
use lua_ffi::ffi::luaL_Reg;
use lua_ffi::lua_method;

use crate::game::scripting::lua::{DRS_RUNTIME_GLOBAL, LuaScriptingState};
use crate::scene::game_scene::GameScene;

pub struct LuaGameScene {
    valid_reference: bool,
    ptr: *mut GameScene,
}

impl LuaGameScene {
    unsafe fn lua_get_tick(&self, state: &mut State) -> c_int {
        state.push((*self.ptr).tick as u32);

        1
    }

    pub(crate) fn new(ptr: *mut GameScene) -> LuaGameScene {
        LuaGameScene { valid_reference: true, ptr }
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
        vec![lua_method!("tick", LuaGameScene, LuaGameScene::lua_get_tick)]
    }
}

impl LuaScriptingState {
    pub fn scene_tick(&mut self) {
        if let Some(state) = &mut self.state {
            let val = LuaGameScene::new(self.game_scene);

            state.get_global(DRS_RUNTIME_GLOBAL);
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
