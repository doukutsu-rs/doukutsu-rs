use num_traits::clamp;

use crate::bitfield;
use crate::caret::CaretType;
use crate::common::{Direction, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::ggez::{Context, GameResult};
use crate::SharedGameState;
use crate::str;

bitfield! {
  pub struct Flags(u32);
  impl Debug;

  pub flag_x01, set_flag_x01: 0;
  pub flag_x02, set_flag_x02: 1;
  pub flag_x04, set_flag_x04: 2;
  pub flag_x08, set_flag_x08: 3;
  pub flag_x10, set_flag_x10: 4;
  pub flag_x20, set_flag_x20: 5;
  pub flag_x40, set_flag_x40: 6;
  pub flag_x80, set_flag_x80: 7;
  pub underwater, set_underwater: 8; // 0x100
  pub flag_x200, set_flag_x200: 9;
  pub flag_x400, set_flag_x400: 10;
  pub flag_x800, set_flag_x800: 11;
  pub force_left, set_force_left: 12; // 0x1000
  pub force_up, set_force_up: 13; // 0x2000
  pub force_right, set_force_right: 14; // 0x4000
  pub force_down, set_force_down: 15; // 0x8000
  pub flag_x10000, set_flag_x10000: 16; // 0x10000
  pub flag_x20000, set_flag_x20000: 17; // 0x20000
  pub flag_x40000, set_flag_x40000: 18; // 0x40000
  pub flag_x80000, set_flag_x80000: 19; // 0x80000

  // engine specific flags
  pub head_bounced, set_head_bounced: 31;
}

bitfield! {
  pub struct Equip(u16);
  impl Debug;

  pub has_booster_0_8, set_booster_0_8: 0;
  pub has_map, set_map: 1;
  pub has_arms_barrier, set_arms_barrier: 2;
  pub has_turbocharge, set_turbocharge: 3;
  pub has_air_tank, set_air_tank: 4;
  pub has_booster_2_0, set_booster_2_0: 5;
  pub has_mimiga_mask, set_mimiga_mask: 6;
  pub has_whimsical_star, set_whimsical_star: 7;
  pub has_nikumaru, set_nikumaru: 8;
  // 7 bits wasted, thx pixel
}


bitfield! {
  pub struct Cond(u16);
  impl Debug;

  pub cond_x01, set_cond_x01: 0;
  pub cond_x02, set_cond_x02: 1;
  pub cond_x04, set_cond_x04: 2;
  pub cond_x08, set_cond_x08: 3;
  pub cond_x10, set_cond_x10: 4;
  pub cond_x20, set_cond_x20: 5;
  pub cond_x40, set_cond_x40: 6;
  pub visible, set_visible: 7;
}

pub struct Player {
    pub x: isize,
    pub y: isize,
    pub vel_x: isize,
    pub vel_y: isize,
    pub target_x: isize,
    pub target_y: isize,
    pub life: usize,
    pub max_life: usize,
    pub cond: Cond,
    pub flags: Flags,
    pub equip: Equip,
    pub direction: Direction,
    pub view: Rect<usize>,
    pub hit: Rect<usize>,
    pub unit: u8,
    pub question: bool,
    pub booster_fuel: usize,
    pub up: bool,
    pub down: bool,
    pub shock_counter: u8,
    index_x: isize,
    index_y: isize,
    sprash: bool,
    booster_switch: u8,
    star: u8,
    bubble: u8,
    exp_wait: isize,
    exp_count: isize,
    anim_num: usize,
    anim_wait: isize,
    anim_rect: Rect<usize>,
    tex_player_name: String,
}

impl Player {
    pub fn new(state: &mut SharedGameState) -> Self {
        let constants = &state.constants;

        let tex_player_name = str!("MyChar");

        Self {
            x: 0,
            y: 0,
            vel_x: 0,
            vel_y: 0,
            target_x: 0,
            target_y: 0,
            life: constants.my_char.life,
            max_life: constants.my_char.max_life,
            cond: Cond(constants.my_char.cond),
            flags: Flags(constants.my_char.flags),
            equip: Equip(constants.my_char.equip),
            direction: constants.my_char.direction,
            view: constants.my_char.view,
            hit: constants.my_char.hit,
            unit: constants.my_char.unit,
            question: false,
            booster_fuel: 0,
            index_x: 0,
            index_y: 0,
            sprash: false,
            up: false,
            down: false,
            shock_counter: 0,
            booster_switch: 0,
            star: 0,
            bubble: 0,
            exp_wait: 0,
            exp_count: 0,
            anim_num: 0,
            anim_wait: 0,
            anim_rect: constants.my_char.animations_right[0],
            tex_player_name,
        }
    }

    fn tick_normal(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.cond.cond_x02() {
            return Ok(());
        }

        let physics = if self.flags.underwater() { state.constants.my_char.water_physics } else { state.constants.my_char.air_physics };

        self.question = false;
        if self.flags.head_bounced() {
            self.flags.set_head_bounced(false);
            // todo: PlaySoundObject(3, SOUND_MODE_PLAY);
            state.create_caret(self.x, self.y - self.hit.top as isize, CaretType::LittleParticles, Direction::Left);
            state.create_caret(self.x, self.y - self.hit.top as isize, CaretType::LittleParticles, Direction::Left);
        }

        if !state.control_flags.control_enabled() {
            self.booster_switch = 0;
        }

        // todo: split those into separate procedures and refactor (try to not break the logic!)

        // ground movement
        if self.flags.flag_x08() || self.flags.flag_x10() || self.flags.flag_x20() {
            self.booster_switch = 0;

            if self.equip.has_booster_0_8() || self.equip.has_booster_2_0() {
                self.booster_fuel = state.constants.booster.fuel;
            } else {
                self.booster_fuel = 0;
            }

            if state.control_flags.control_enabled() {
                if state.key_trigger.only_down() && state.key_state.only_down() && !self.cond.cond_x01() && !state.control_flags.flag_x04() {
                    self.cond.set_cond_x01(true);
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
            self.down = state.key_state.down() && !self.flags.flag_x08();

            if state.key_trigger.jump() && (self.flags.flag_x08() || self.flags.flag_x10() || self.flags.flag_x20()) && !self.flags.force_up() {
                self.vel_y = -physics.jump;
                // todo: PlaySoundObject(15, SOUND_MODE_PLAY);
            }
        }

        // stop interacting when moved
        if state.control_flags.control_enabled() && (state.key_state.left() || state.key_state.right() || state.key_state.up() || state.key_state.jump() || state.key_state.fire()) {
            self.cond.set_cond_x01(false);
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
                    if self.flags.flag_x01() || self.flags.flag_x04() {
                        self.vel_y = -0x100; // -0.5fix9
                    }

                    if self.direction == Direction::Left {
                        self.vel_x -= 0x20; // 0.1fix9
                    }
                    if self.direction == Direction::Right {
                        self.vel_x += 0x20; // 0.1fix9
                    }

                    // todo: sound
                    if state.key_trigger.jump() || self.booster_fuel % 3 == 1 {
                        if self.direction == Direction::Left || self.direction == Direction::Right {
                            state.create_caret(self.x + 0x400, self.y + 0x400, CaretType::Exhaust, self.direction.opposite());
                        }
                        // PlaySoundObject(113, SOUND_MODE_PLAY);
                    }
                }
                2 => {
                    self.vel_y -= 0x20;

                    // todo: sound
                    if state.key_trigger.jump() || self.booster_fuel % 3 == 1 {
                        state.create_caret(self.x, self.y + 6 * 0x200, CaretType::Exhaust, Direction::Bottom);
                        // PlaySoundObject(113, SOUND_MODE_PLAY);
                    }
                }
                // todo: sound
                3 if state.key_trigger.jump() || self.booster_fuel % 3 == 1 => {
                    state.create_caret(self.x, self.y + 6 * 0x200, CaretType::Exhaust, Direction::Up);
                    // PlaySoundObject(113, SOUND_MODE_PLAY);
                }
                _ => {}
            }
        } else if self.flags.force_up() {
            self.vel_y += physics.gravity_air;
        } else if self.equip.has_booster_0_8() && self.booster_switch != 0 && self.vel_y > -0x400 {
            self.vel_y -= 0x20;

            if self.booster_fuel % 3 == 0 {
                state.create_caret(self.x, self.y + self.hit.bottom as isize / 2, CaretType::Exhaust, Direction::Bottom);
                // PlaySoundObject(113, SOUND_MODE_PLAY);
            }

            // bounce off of ceiling
            if self.flags.flag_x02() {
                self.vel_y = 0x200; // 1.0fix9
            }
        } else if self.vel_y < 0 && state.control_flags.control_enabled() && state.key_state.jump() {
            self.vel_y += physics.gravity_air;
        } else {
            self.vel_y += physics.gravity_ground;
        }

        if !state.control_flags.control_enabled() || !state.key_trigger.jump() {
            if self.flags.flag_x10() && self.vel_x < 0 {
                self.vel_y = -self.vel_x;
            }

            if self.flags.flag_x20() && self.vel_x > 0 {
                self.vel_y = self.vel_x;
            }

            if (self.flags.flag_x08() && self.flags.flag_x80000() && self.vel_x < 0)
                || (self.flags.flag_x08() && self.flags.flag_x10000() && self.vel_x > 0)
                || (self.flags.flag_x08() && self.flags.flag_x20000() && self.flags.flag_x40000()) {
                self.vel_y = 0x400; // 2.0fix9
            }
        }

        let max_move = if self.flags.underwater() && !(self.flags.force_left() || self.flags.force_up() || self.flags.force_right() || self.flags.force_down()) {
            state.constants.my_char.water_physics.max_move
        } else {
            state.constants.my_char.air_physics.max_move
        };

        self.vel_x = clamp(self.vel_x, -max_move, max_move);
        self.vel_y = clamp(self.vel_y, -max_move, max_move);

        // todo: water splashing

        if !self.flags.underwater() {
            self.sprash = false;
        }

        // spike damage
        if self.flags.flag_x400() {
            self.damage(10);
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

        self.target_x = self.x + self.index_x;
        self.target_y = self.y + self.index_y;

        if self.vel_x > physics.resist || self.vel_x < -physics.resist {
            self.x += self.vel_x;
        }

        self.y += self.vel_y;

        Ok(())
    }

    fn tick_stream(&mut self, state: &mut SharedGameState) -> GameResult {
        Ok(())
    }

    fn tick_animation(&mut self, state: &SharedGameState) {
        if self.cond.cond_x02() {
            return;
        }

        if self.flags.flag_x08() {
            if self.cond.cond_x01() {
                self.anim_num = 11;
            } else if state.control_flags.control_enabled() && state.key_state.up() && (state.key_state.left() || state.key_state.right()) {
                self.cond.set_cond_x04(true);

                self.anim_wait += 1;
                if self.anim_wait > 4 {
                    self.anim_wait = 0;

                    self.anim_num += 1;
                    if self.anim_num == 7 || self.anim_num == 9 {
                        // PlaySoundObject(24, SOUND_MODE_PLAY); todo
                    }
                }

                if self.anim_num > 9 || self.anim_num < 6 {
                    self.anim_num = 6;
                }
            } else if state.control_flags.control_enabled() && (state.key_state.left() || state.key_state.right()) {
                self.cond.set_cond_x04(true);

                self.anim_wait += 1;
                if self.anim_wait > 4 {
                    self.anim_wait = 0;

                    self.anim_num += 1;
                    if self.anim_num == 2 || self.anim_num == 4 {
                        // PlaySoundObject(24, SOUND_MODE_PLAY); todo
                    }
                }

                if self.anim_num > 4 || self.anim_num < 1 {
                    self.anim_num = 1;
                }
            } else if state.control_flags.control_enabled() && state.key_state.up() {
                if self.cond.cond_x04() {
                    // PlaySoundObject(24, SOUND_MODE_PLAY); todo
                }

                self.cond.set_cond_x04(false);
                self.anim_num = 5;
            } else {
                if self.cond.cond_x04() {
                    // PlaySoundObject(24, SOUND_MODE_PLAY); todo
                }

                self.cond.set_cond_x04(false);
                self.anim_num = 0;
            }
        } else if state.key_state.up() {
            self.anim_num = 6;
        } else if state.key_state.down() {
            self.anim_num = 10;
        } else {
            self.anim_num = if self.vel_y > 0 { 1 } else { 3 };
        }

        match self.direction {
            Direction::Left => {
                self.anim_rect = state.constants.my_char.animations_left[self.anim_num];
            }
            Direction::Right => {
                self.anim_rect = state.constants.my_char.animations_right[self.anim_num];
            }
            _ => {}
        }
    }

    pub fn damage(&mut self, hp: isize) {
        if self.shock_counter > 0 {
            return;
        }

        // PlaySoundObject(16, SOUND_MODE_PLAY); // todo: damage sound
        self.shock_counter = 128;
        self.cond.set_cond_x01(false);

        if self.unit != 1 {
            self.vel_y = -0x400; // -2.0fix9
        }

        self.life = if hp >= self.life as isize { 0 } else { (self.life as isize - hp) as usize };

        if self.equip.has_whimsical_star() && self.star > 0 {
            self.star -= 1;
        }
    }
}

impl GameEntity for Player {
    fn tick(&mut self, state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        if !self.cond.visible() {
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

        match self.unit {
            0 => {
                if state.control_flags.flag_x04() && state.control_flags.control_enabled() {
                    // AirProcess(); // todo
                }

                self.tick_normal(state)?;
            }
            1 => {
                self.tick_stream(state)?;
            }
            _ => {}
        }

        self.cond.set_cond_x20(false);
        self.tick_animation(state);

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult<()> {
        if !self.cond.visible() || self.cond.cond_x02() {
            return Ok(());
        }

        // todo draw weapon
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_player_name)?;
        batch.add_rect(
            (((self.x - self.view.left as isize) / 0x200) - (frame.x / 0x200)) as f32,
            (((self.y - self.view.top as isize) / 0x200) - (frame.y / 0x200)) as f32,
            &self.anim_rect,
        );
        batch.draw(ctx)?;

        Ok(())
    }
}
