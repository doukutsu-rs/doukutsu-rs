use std::clone::Clone;

use num_derive::FromPrimitive;
use num_traits::{clamp, FromPrimitive};

use crate::caret::CaretType;
use crate::common::{Condition, Equipment, Flag};
use crate::common::{Direction, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::ggez::{Context, GameResult};
use crate::inventory::Inventory;
use crate::shared_game_state::SharedGameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum ControlMode {
    Normal = 0,
    IronHead,
}

#[derive(Clone)]
pub struct Player {
    pub x: isize,
    pub y: isize,
    pub vel_x: isize,
    pub vel_y: isize,
    pub target_x: isize,
    pub target_y: isize,
    pub life: u16,
    pub max_life: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub equip: Equipment,
    pub direction: Direction,
    pub display_bounds: Rect<usize>,
    pub hit_bounds: Rect<usize>,
    pub control_mode: ControlMode,
    pub question: bool,
    pub booster_fuel: usize,
    pub up: bool,
    pub down: bool,
    pub shock_counter: u8,
    pub current_weapon: u8,
    pub update_target: bool,
    pub stars: u8,
    weapon_offset_y: i8,
    index_x: isize,
    index_y: isize,
    splash: bool,
    booster_switch: u8,
    bubble: u8,
    exp_wait: isize,
    exp_count: isize,
    anim_num: u16,
    anim_counter: u16,
    anim_rect: Rect<usize>,
    weapon_rect: Rect<usize>,
}

impl Player {
    pub fn new(state: &mut SharedGameState) -> Player {
        let constants = &state.constants;

        Player {
            x: 0,
            y: 0,
            vel_x: 0,
            vel_y: 0,
            target_x: 0,
            target_y: 0,
            life: constants.my_char.life,
            max_life: constants.my_char.max_life,
            cond: Condition(0x80),
            flags: Flag(0),
            equip: Equipment(0),
            direction: Direction::Right,
            display_bounds: constants.my_char.display_bounds,
            hit_bounds: constants.my_char.hit_bounds,
            control_mode: constants.my_char.control_mode,
            question: false,
            booster_fuel: 0,
            index_x: 0,
            index_y: 0,
            splash: false,
            update_target: true,
            up: false,
            down: false,
            current_weapon: 0,
            weapon_offset_y: 0,
            shock_counter: 0,
            booster_switch: 0,
            stars: 0,
            bubble: 0,
            exp_wait: 0,
            exp_count: 0,
            anim_num: 0,
            anim_counter: 0,
            anim_rect: constants.my_char.animations_right[0],
            weapon_rect: Rect::new(0, 0, 0, 0),
        }
    }

    fn tick_normal(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.cond.hidden() {
            return Ok(());
        }

        let physics = if self.flags.in_water() { state.constants.my_char.water_physics } else { state.constants.my_char.air_physics };

        self.question = false;

        if !state.control_flags.control_enabled() {
            self.booster_switch = 0;
        }

        // todo: split those into separate procedures and refactor (try to not break the logic!)

        // ground movement
        if self.flags.hit_bottom_wall() || self.flags.hit_right_slope() || self.flags.hit_left_slope() {
            self.booster_switch = 0;

            if self.equip.has_booster_0_8() || self.equip.has_booster_2_0() {
                self.booster_fuel = state.constants.booster.fuel;
            } else {
                self.booster_fuel = 0;
            }

            if state.control_flags.control_enabled() {
                if state.key_trigger.only_down() && state.key_state.only_down() && !self.cond.interacted() && !state.control_flags.interactions_disabled() {
                    self.cond.set_interacted(true);
                    self.question = true;
                } else {
                    if state.key_state.left() && self.vel_x > -physics.max_dash {
                        self.vel_x -= physics.dash_ground;
                    }

                    if state.key_state.right() && self.vel_x < physics.max_dash {
                        self.vel_x += physics.dash_ground;
                    }

                    if state.key_state.left() {
                        self.direction = Direction::Left;
                    }

                    if state.key_state.right() {
                        self.direction = Direction::Right;
                    }
                }
            }

            if !self.cond.cond_x20() {
                if self.vel_x < 0 {
                    if self.vel_x > -physics.resist {
                        self.vel_x = 0;
                    } else {
                        self.vel_x += physics.resist;
                    }
                }
                if self.vel_x > 0 {
                    if self.vel_x < physics.resist {
                        self.vel_x = 0;
                    } else {
                        self.vel_x -= physics.resist;
                    }
                }
            }
        } else { // air movement
            if state.control_flags.control_enabled() {
                if state.key_trigger.jump() && self.booster_fuel != 0 {
                    if self.equip.has_booster_0_8() {
                        self.booster_switch = 1;

                        if self.vel_y > 0x100 { // 0.5fix9
                            self.vel_y /= 2;
                        }
                    } else if self.equip.has_booster_2_0() {
                        if state.key_state.up() {
                            self.booster_switch = 2;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_up;
                        } else if state.key_state.left() {
                            self.booster_switch = 1;
                            self.vel_x = state.constants.booster.b2_0_left;
                            self.vel_y = 0;
                        } else if state.key_state.right() {
                            self.booster_switch = 1;
                            self.vel_x = state.constants.booster.b2_0_right;
                            self.vel_y = 0;
                        } else if state.key_state.down() {
                            self.booster_switch = 3;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_down;
                        } else {
                            self.booster_switch = 2;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_up_nokey;
                        }
                    }
                }

                if state.key_state.left() && self.vel_x > -physics.max_dash {
                    self.vel_x -= physics.dash_air;
                }

                if state.key_state.right() && self.vel_x < physics.max_dash {
                    self.vel_x += physics.dash_air;
                }

                if state.key_state.left() {
                    self.direction = Direction::Left;
                }

                if state.key_state.right() {
                    self.direction = Direction::Right;
                }
            }

            if self.equip.has_booster_2_0() && self.booster_switch != 0 && (!state.key_state.jump() || self.booster_fuel == 0) {
                match self.booster_switch {
                    1 => { self.vel_x /= 2 }
                    2 => { self.vel_y /= 2 }
                    _ => {}
                }
            }

            if self.booster_fuel == 0 || !state.key_state.jump() {
                self.booster_switch = 0;
            }
        }

        // jumping
        if state.control_flags.control_enabled() {
            self.up = state.key_state.up();
            self.down = state.key_state.down() && !self.flags.hit_bottom_wall();

            if state.key_trigger.jump() && (self.flags.hit_bottom_wall() || self.flags.hit_right_slope() || self.flags.hit_left_slope()) && !self.flags.force_up() {
                self.vel_y = -physics.jump;
                state.sound_manager.play_sfx(15);
            }
        }

        // stop interacting when moved
        if state.control_flags.control_enabled() && (state.key_state.left() || state.key_state.right() || state.key_state.up() || state.key_state.jump() || state.key_state.fire()) {
            self.cond.set_interacted(false);
        }

        // booster losing fuel
        if self.booster_switch != 0 && self.booster_fuel != 0 {
            self.booster_fuel -= 1;
        }

        // wind / current forces

        if self.flags.force_left() {
            self.vel_x -= 0x88;
        }
        if self.flags.force_up() {
            self.vel_y -= 0x80;
        }
        if self.flags.force_right() {
            self.vel_x += 0x80;
        }
        if self.flags.force_down() {
            self.vel_y += 0x55;
        }

        if self.equip.has_booster_2_0() && self.booster_switch != 0 {
            match self.booster_switch {
                1 => {
                    if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                        self.vel_y = -0x100; // -0.5fix9
                    }

                    if self.direction == Direction::Left {
                        self.vel_x -= 0x20; // 0.1fix9
                    }
                    if self.direction == Direction::Right {
                        self.vel_x += 0x20; // 0.1fix9
                    }

                    if state.key_trigger.jump() || self.booster_fuel % 3 == 1 {
                        if self.direction == Direction::Left || self.direction == Direction::Right {
                            state.create_caret(self.x + 0x400, self.y + 0x400, CaretType::Exhaust, self.direction.opposite());
                        }
                        state.sound_manager.play_sfx(113);
                    }
                }
                2 => {
                    self.vel_y -= 0x20;

                    if state.key_trigger.jump() || self.booster_fuel % 3 == 1 {
                        state.create_caret(self.x, self.y + 6 * 0x200, CaretType::Exhaust, Direction::Bottom);
                        state.sound_manager.play_sfx(113);
                    }
                }
                3 if state.key_trigger.jump() || self.booster_fuel % 3 == 1 => {
                    state.create_caret(self.x, self.y + 6 * 0x200, CaretType::Exhaust, Direction::Up);
                    state.sound_manager.play_sfx(113);
                }
                _ => {}
            }
        } else if self.flags.force_up() {
            self.vel_y += physics.gravity_air;
        } else if self.equip.has_booster_0_8() && self.booster_switch != 0 && self.vel_y > -0x400 {
            self.vel_y -= 0x20;

            if self.booster_fuel % 3 == 0 {
                state.create_caret(self.x, self.y + self.hit_bounds.bottom as isize / 2, CaretType::Exhaust, Direction::Bottom);
                state.sound_manager.play_sfx(113);
            }

            // bounce off of ceiling
            if self.flags.hit_top_wall() {
                self.vel_y = 0x200; // 1.0fix9
            }
        } else if self.vel_y < 0 && state.control_flags.control_enabled() && state.key_state.jump() {
            self.vel_y += physics.gravity_air;
        } else {
            self.vel_y += physics.gravity_ground;
        }

        if !state.control_flags.control_enabled() || !state.key_trigger.jump() {
            if self.flags.hit_right_slope() && self.vel_x < 0 {
                self.vel_y = -self.vel_x;
            }

            if self.flags.hit_left_slope() && self.vel_x > 0 {
                self.vel_y = self.vel_x;
            }

            if (self.flags.hit_bottom_wall() && self.flags.hit_right_bigger_half() && self.vel_x < 0)
                || (self.flags.hit_bottom_wall() && self.flags.hit_left_bigger_half() && self.vel_x > 0)
                || (self.flags.hit_bottom_wall() && self.flags.hit_left_smaller_half() && self.flags.hit_right_smaller_half()) {
                self.vel_y = 0x400; // 2.0fix9
            }
        }

        let max_move = if self.flags.in_water() && !(self.flags.force_left() || self.flags.force_up() || self.flags.force_right() || self.flags.force_down()) {
            state.constants.my_char.water_physics.max_move
        } else {
            state.constants.my_char.air_physics.max_move
        };

        self.vel_x = clamp(self.vel_x, -max_move, max_move);
        self.vel_y = clamp(self.vel_y, -max_move, max_move);

        // todo: water splashing

        if !self.flags.in_water() {
            self.splash = false;
        }

        // spike damage
        if self.flags.hit_by_spike() {
            self.damage(10, state);
        }

        // camera
        if self.direction == Direction::Left {
            self.index_x -= 0x200; // 1.0fix9
            if self.index_x < -0x8000 { // -64.0fix9
                self.index_x = -0x8000;
            }
        } else { // possible bug?
            self.index_x += 0x200; // 1.0fix9
            if self.index_x > 0x8000 { // -64.0fix9
                self.index_x = 0x8000;
            }
        }

        if state.control_flags.control_enabled() && state.key_state.up() {
            self.index_y -= 0x200; // 1.0fix9
            if self.index_y < -0x8000 { // -64.0fix9
                self.index_y = -0x8000;
            }
        } else if state.control_flags.control_enabled() && state.key_state.down() {
            self.index_y += 0x200; // 1.0fix9
            if self.index_y > 0x8000 { // -64.0fix9
                self.index_y = 0x8000;
            }
        } else {
            if self.index_y > 0x200 { // 1.0fix9
                self.index_y -= 0x200;
            }

            if self.index_y < -0x200 { // 1.0fix9
                self.index_y += 0x200;
            }
        }

        if self.update_target {
            self.target_x = self.x + self.index_x;
            self.target_y = self.y + self.index_y;
        }

        if self.vel_x > physics.resist || self.vel_x < -physics.resist {
            self.x += self.vel_x;
        }

        self.y += self.vel_y;

        Ok(())
    }

    fn tick_ironhead(&mut self, state: &mut SharedGameState) -> GameResult {
        // todo ironhead boss controls
        Ok(())
    }

    fn tick_animation(&mut self, state: &mut SharedGameState) {
        if self.cond.hidden() {
            return;
        }

        if self.flags.hit_bottom_wall() {
            if self.cond.interacted() {
                self.anim_num = 11;
            } else if state.control_flags.control_enabled() && state.key_state.up() && (state.key_state.left() || state.key_state.right()) {
                self.cond.set_fallen(true);

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num == 7 || self.anim_num == 9 {
                        state.sound_manager.play_sfx(24);
                    }
                }

                if self.anim_num > 9 || self.anim_num < 6 {
                    self.anim_num = 6;
                }
            } else if state.control_flags.control_enabled() && (state.key_state.left() || state.key_state.right()) {
                self.cond.set_fallen(true);

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num == 2 || self.anim_num == 4 {
                        state.sound_manager.play_sfx(24);
                    }
                }

                if self.anim_num > 4 || self.anim_num < 1 {
                    self.anim_num = 1;
                }
            } else if state.control_flags.control_enabled() && state.key_state.up() {
                if self.cond.fallen() {
                    state.sound_manager.play_sfx(24);
                }

                self.cond.set_fallen(false);
                self.anim_num = 5;
            } else {
                if self.cond.fallen() {
                    state.sound_manager.play_sfx(24);
                }

                self.cond.set_fallen(false);
                self.anim_num = 0;
            }
        } else if state.key_state.up() {
            self.anim_num = 6;
        } else if state.key_state.down() {
            self.anim_num = 10;
        } else {
            self.anim_num = if self.vel_y > 0 { 1 } else { 3 };
        }

        self.weapon_offset_y = 0;
        self.weapon_rect.left = (self.current_weapon as usize % 13) * 24;
        self.weapon_rect.top = (self.current_weapon as usize / 13) * 96;
        self.weapon_rect.right = self.weapon_rect.left + 24;
        self.weapon_rect.bottom = self.weapon_rect.top + 16;

        match self.direction {
            Direction::Left => {
                self.anim_rect = state.constants.my_char.animations_left[self.anim_num as usize];
            }
            Direction::Right => {
                self.weapon_rect.top += 16;
                self.weapon_rect.bottom += 16;
                self.anim_rect = state.constants.my_char.animations_right[self.anim_num as usize];
            }
            _ => {}
        }

        if self.up {
            self.weapon_offset_y = -4;
            self.weapon_rect.top += 32;
            self.weapon_rect.bottom += 32;
        } else if self.down {
            self.weapon_offset_y = 4;
            self.weapon_rect.top += 64;
            self.weapon_rect.bottom += 64;
        }

        if self.anim_num == 1 || self.anim_num == 3 || self.anim_num == 6 || self.anim_num == 8 {
            self.weapon_rect.top += 1;
        }
    }

    pub fn damage(&mut self, hp: isize, state: &mut SharedGameState) {
        if state.god_mode || self.shock_counter > 0 {
            return;
        }

        state.sound_manager.play_sfx(16);
        self.shock_counter = 128;
        self.cond.set_interacted(false);

        if self.control_mode == ControlMode::Normal {
            self.vel_y = -0x400; // -2.0fix9
        }

        self.life = self.life.saturating_sub(hp as u16);

        if self.equip.has_whimsical_star() && self.stars > 0 {
            self.stars -= 1;
        }

        if self.life == 0 {
            state.sound_manager.play_sfx(17);
            self.cond.0 = 0;
            state.textscript_vm.start_script(40);
        }
    }
}

impl GameEntity<()> for Player {
    fn tick(&mut self, state: &mut SharedGameState, _cust: ()) -> GameResult {
        if !self.cond.alive() {
            return Ok(());
        }

        if self.exp_wait != 0 {
            self.exp_wait -= 1;
        }

        if self.shock_counter != 0 {
            self.shock_counter -= 1;
        } else if self.exp_count != 0 {
            // SetValueView(&self.x, &self.y, self.exp_count); // todo: damage popup
            self.exp_count = 0;
        }

        // todo: add additional control modes like NXEngine has such as noclip?
        match self.control_mode {
            ControlMode::Normal => {
                if state.control_flags.interactions_disabled() && state.control_flags.control_enabled() {
                    // AirProcess(); // todo
                }

                self.tick_normal(state)?;
            }
            ControlMode::IronHead => {
                self.tick_ironhead(state)?;
            }
        }

        self.cond.set_cond_x20(false);
        self.tick_animation(state);

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult<> {
        if !self.cond.alive() || self.cond.hidden() || (self.shock_counter / 2 % 2 != 0) {
            return Ok(());
        }

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "MyChar")?;
            batch.add_rect(
                (((self.x - self.display_bounds.left as isize) / 0x200) - (frame.x / 0x200)) as f32,
                (((self.y - self.display_bounds.top as isize) / 0x200) - (frame.y / 0x200)) as f32,
                &self.anim_rect,
            );
            batch.draw(ctx)?;
        }

        if self.current_weapon != 0 {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Arms")?;
            match self.direction {
                Direction::Left => {
                    batch.add_rect(
                        (((self.x - self.display_bounds.left as isize) / 0x200) - (frame.x / 0x200)) as f32 - 8.0,
                        (((self.y - self.display_bounds.top as isize) / 0x200) - (frame.y / 0x200)) as f32 + self.weapon_offset_y as f32,
                        &self.weapon_rect,
                    );
                }
                Direction::Right => {
                    batch.add_rect(
                        (((self.x - self.display_bounds.left as isize) / 0x200) - (frame.x / 0x200)) as f32,
                        (((self.y - self.display_bounds.top as isize) / 0x200) - (frame.y / 0x200)) as f32 + self.weapon_offset_y as f32,
                        &self.weapon_rect,
                    );
                }
                _ => {}
            }

            batch.draw(ctx)?;
        }

        Ok(())
    }
}
