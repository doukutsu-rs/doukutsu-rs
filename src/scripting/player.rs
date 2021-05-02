use lua_ffi::{c_int, LuaObject, State};
use lua_ffi::ffi::luaL_Reg;

use crate::inventory::Inventory;
use crate::player::Player;
use crate::scripting::REF_ERROR;
use crate::weapon::WeaponType;

pub struct LuaPlayer {
    valid_reference: bool,
    plr_ptr: *mut Player,
    inv_ptr: *mut Inventory,
}

#[allow(unused)]
impl LuaPlayer {
    fn check_ref(&self, state: &mut State) -> bool {
        if !self.valid_reference {
            state.error(REF_ERROR);
            return true;
        }

        false
    }

    fn lua_get_x(&self, state: &mut State) -> c_int {
        if self.check_ref(state) { return 0; }

        unsafe {
            state.push((*self.plr_ptr).x);
        }

        1
    }

    fn lua_get_y(&self, state: &mut State) -> c_int {
        if self.check_ref(state) { return 0; }

        unsafe {
            state.push((*self.plr_ptr).y);
        }

        1
    }

    fn lua_get_vel_x(&self, state: &mut State) -> c_int {
        if self.check_ref(state) { return 0; }

        unsafe {
            state.push((*self.plr_ptr).vel_x);
        }

        1
    }

    fn lua_get_vel_y(&self, state: &mut State) -> c_int {
        if self.check_ref(state) { return 0; }

        unsafe {
            state.push((*self.plr_ptr).vel_y);
        }

        1
    }

    fn lua_set_vel_x(&self, state: &mut State) -> c_int {
        if self.check_ref(state) { return 0; }

        unsafe {
            if let Some(vel_x) = state.to_int(2) {
                (*self.plr_ptr).vel_x = vel_x;
            }
        }

        0
    }

    fn lua_set_vel_y(&self, state: &mut State) -> c_int {
        if self.check_ref(state) { return 0; }

        unsafe {
            if let Some(vel_y) = state.to_int(2) {
                (*self.plr_ptr).vel_y = vel_y;
            }
        }

        0
    }

    fn lua_get_weapon_ammo(&self, state: &mut State) -> c_int {
        if self.check_ref(state) { return 0; }

        if let Some(index) = state.to_int(2) {} else {
            state.error("Weapon type must be a number");
            return 0;
        }

        unsafe {
            if let Some(weap) = (*self.inv_ptr).get_weapon_by_type_mut(WeaponType::PolarStar) {}
        }

        1
    }

    pub(crate) fn new(plr_ptr: *mut Player, inv_ptr: *mut Inventory) -> LuaPlayer {
        LuaPlayer {
            valid_reference: true,
            plr_ptr,
            inv_ptr,
        }
    }
}

impl Drop for LuaPlayer {
    fn drop(&mut self) {
        self.valid_reference = false;
    }
}

impl LuaObject for LuaPlayer {
    fn name() -> *const i8 {
        c_str!("Player")
    }

    fn lua_fns() -> Vec<luaL_Reg> {
        vec![
            lua_method!("x", LuaPlayer, LuaPlayer::lua_get_x),
            lua_method!("y", LuaPlayer, LuaPlayer::lua_get_y),
            lua_method!("velX", LuaPlayer, LuaPlayer::lua_get_vel_x),
            lua_method!("velX", LuaPlayer, LuaPlayer::lua_get_vel_y),
            lua_method!("setVelX", LuaPlayer, LuaPlayer::lua_set_vel_x),
            lua_method!("setVelY", LuaPlayer, LuaPlayer::lua_set_vel_y),
        ]
    }
}
