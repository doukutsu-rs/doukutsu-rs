use ggez::{Context, GameResult};
use num_traits::clamp;

use crate::bitfield;
use crate::common::{Direction, Rect};
use crate::engine_constants::PhysicsConsts;
use crate::entity::GameEntity;
use crate::frame::Frame;
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
    pub xm: isize,
    pub ym: isize,
    pub target_x: isize,
    pub target_y: isize,
    pub cond: Cond,
    pub flags: Flags,
    pub equip: Equip,
    pub direction: Direction,
    pub view: Rect<usize>,
    pub hit: Rect<usize>,
    pub life: u16,
    pub max_life: u16,
    pub unit: u8,
    pub air_physics: PhysicsConsts,
    pub water_physics: PhysicsConsts,
    index_x: isize,
    index_y: isize,
    sprash: bool,
    ques: bool,
    up: bool,
    down: bool,
    shock: u8,
    bubble: u8,
    boost_sw: u8,
    boost_cnt: isize,
    exp_wait: isize,
    exp_count: isize,
    anim_num: usize,
    anim_wait: isize,
    anim_rect: Rect<usize>,
    tex_player_name: String,
}

impl Player {
    pub fn new(state: &mut SharedGameState, ctx: &mut Context) -> GameResult<Player> {
        let constants = &state.constants;

        let tex_player_name = str!("MyChar");
        state.texture_set.ensure_texture_loaded(ctx, &tex_player_name)?;

        Ok(Player {
            x: 0,
            y: 0,
            xm: 0,
            ym: 0,
            target_x: 0,
            target_y: 0,
            cond: Cond(constants.my_char.cond),
            flags: Flags(constants.my_char.flags),
            equip: Equip(constants.my_char.equip),
            direction: constants.my_char.direction.clone(),
            view: constants.my_char.view,
            hit: constants.my_char.hit,
            life: constants.my_char.life,
            max_life: constants.my_char.max_life,
            unit: constants.my_char.unit,
            air_physics: constants.my_char.air_physics,
            water_physics: constants.my_char.water_physics,
            index_x: 0,
            index_y: 0,
            sprash: false,
            ques: false,
            up: false,
            down: false,
            shock: 0,
            bubble: 0,
            boost_sw: 0,
            boost_cnt: 0,
            exp_wait: 0,
            exp_count: 0,
            anim_num: 0,
            anim_wait: 0,
            anim_rect: constants.my_char.animations_right[0],
            tex_player_name,
        })
    }

    fn tick_normal(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult<()> {
        if self.cond.cond_x02() {
            return Ok(());
        }

        let physics = if self.flags.underwater() { &self.water_physics } else { &self.air_physics };

        self.ques = false;

        if !state.flags.control_enabled() {
            self.boost_sw = 0;
        }

        // todo: split those into separate procedures and refactor (try to not break the logic!)

        // ground movement
        if self.flags.flag_x08() || self.flags.flag_x10() || self.flags.flag_x20() {
            self.boost_sw = 0;

            if self.equip.has_booster_0_8() || self.equip.has_booster_2_0() {
                self.boost_cnt = 50;
            } else {
                self.boost_cnt = 0;
            }

            if state.flags.control_enabled() {
                if state.key_trigger.only_down() && state.key_state.only_down() && !self.cond.cond_x01() && state.flags.flag_x04() {
                    self.cond.set_cond_x01(true);
                    self.ques = true;
                } else {
                    if state.key_state.left() && self.xm > -physics.max_dash {
                        self.xm -= physics.dash_ground;
                    }

                    if state.key_state.right() && self.xm < physics.max_dash {
                        self.xm += physics.dash_ground;
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
                if self.xm < 0 {
                    if self.xm > -physics.resist {
                        self.xm = 0;
                    } else {
                        self.xm += physics.resist;
                    }
                } else if self.xm > 0 {
                    if self.xm < physics.resist {
                        self.xm = 0;
                    } else {
                        self.xm -= physics.resist;
                    }
                }
            }
        } else { // air movement
            if state.flags.control_enabled() {
                if (self.equip.has_booster_0_8() || self.equip.has_booster_2_0()) && state.key_trigger.jump() && self.boost_cnt != 0 {
                    if self.equip.has_booster_0_8() {
                        self.boost_sw = 1;

                        if self.ym > 0x100 { // 0.5fix9
                            self.ym /= 2;
                        }
                    }

                    if self.equip.has_booster_2_0() {
                        if state.key_state.up() {
                            self.boost_sw = 2;
                            self.xm = 0;
                            self.ym = state.constants.booster.b2_0_up;
                        } else if state.key_state.left() {
                            self.boost_sw = 2;
                            self.xm = 0;
                            self.ym = state.constants.booster.b2_0_left;
                        } else if state.key_state.right() {
                            self.boost_sw = 2;
                            self.xm = 0;
                            self.ym = state.constants.booster.b2_0_right;
                        } else if state.key_state.down() {
                            self.boost_sw = 2;
                            self.xm = 0;
                            self.ym = state.constants.booster.b2_0_down;
                        } else {
                            self.boost_sw = 2;
                            self.xm = 0;
                            self.ym = state.constants.booster.b2_0_up_nokey;
                        }
                    }
                }

                if state.key_state.left() && self.xm > -physics.max_dash {
                    self.xm -= physics.dash_air;
                }

                if state.key_state.right() && self.xm < physics.max_dash {
                    self.xm += physics.dash_air;
                }

                if state.key_state.left() {
                    self.direction = Direction::Left;
                }

                if state.key_state.right() {
                    self.direction = Direction::Right;
                }
            }

            if self.equip.has_booster_2_0() && self.boost_sw != 0 && !state.key_state.jump() || self.boost_cnt == 0 {
                match self.boost_sw {
                    1 => { self.xm /= 2 }
                    2 => { self.ym /= 2 }
                    _ => {}
                }
            }

            if self.boost_cnt == 0 || !state.key_state.jump() {
                self.boost_cnt = 0;
            }
        }

        // jumping
        if state.flags.control_enabled() {
            self.up = state.key_state.up();
            self.down = state.key_state.down() && !self.flags.flag_x08();

            if state.key_trigger.jump() && (self.flags.flag_x08() || self.flags.flag_x10() || self.flags.flag_x20()) {
                if !self.flags.force_up() {
                    self.ym = -physics.jump;
                    // todo: PlaySoundObject(15, SOUND_MODE_PLAY);
                }
            }
        }

        // stop interacting when moved
        if state.flags.control_enabled() && (state.key_state.left() || state.key_state.right() || state.key_state.up() || state.key_state.jump() || state.key_state.fire()) {
            self.cond.set_cond_x01(false);
        }

        // booster losing fuel
        if self.boost_sw != 0 && self.boost_cnt != 0 {
            self.boost_cnt -= 1;
        }

        // wind / current forces

        if self.flags.force_left() {
            self.xm -= 0x88;
        }
        if self.flags.force_up() {
            self.ym -= 0x80;
        }
        if self.flags.force_right() {
            self.xm += 0x80;
        }
        if self.flags.force_down() {
            self.ym += 0x55;
        }

        if self.equip.has_booster_2_0() && self.boost_sw != 0 {
            match self.boost_sw {
                1 => {
                    if self.flags.flag_x01() || self.flags.flag_x04() {
                        self.ym = -0x100; // -0.5fix9
                    }

                    if self.direction == Direction::Left {
                        self.xm -= 0x20; // 0.1fix9
                    }
                    if self.direction == Direction::Right {
                        self.xm += 0x20; // 0.1fix9
                    }

                    // todo: particles and sound
                    if state.key_trigger.jump() || self.boost_cnt % 3 == 1 {
                        if self.direction == Direction::Left {
                            // SetCaret(self.x + 2 * 0x200, self.y + 2 * 0x200, 7, 2);
                        }
                        if self.direction == Direction::Right {
                            // SetCaret(self.x + 2 * 0x200, self.y + 2 * 0x200, 7, 0);
                        }
                        // PlaySoundObject(113, SOUND_MODE_PLAY);
                    }
                }
                2 => {
                    self.ym -= 0x20;

                    // todo: particles and sound
                    if state.key_trigger.jump() || self.boost_cnt % 3 == 1 {
                        // SetCaret(self.x, self.y + 6 * 0x200, 7, 3);
                        // PlaySoundObject(113, SOUND_MODE_PLAY);
                    }
                }
                // todo: particles and sound
                3 if state.key_trigger.jump() || self.boost_cnt % 3 == 1 => {
                    // SetCaret(self.x, self.y + 6 * 0x200, 7, 1);
                    // PlaySoundObject(113, SOUND_MODE_PLAY);
                }
                _ => {}
            }
        } else if self.flags.force_up() {
            self.ym += physics.gravity_air;
        } else if self.equip.has_booster_0_8() && self.boost_sw != 0 && self.ym > -0x400 {
            self.ym -= 0x20;

            if self.boost_cnt % 3 == 0 {
                // SetCaret(self.x, self.y + self.hit.bottom as isize / 2, 7, 3);
                // PlaySoundObject(113, SOUND_MODE_PLAY);
            }

            // bounce off of ceiling
            if self.flags.flag_x02() {
                self.ym = 0x200; // 1.0fix9
            }
        } else if self.ym < 0 && state.flags.control_enabled() && state.key_state.jump() {
            self.ym += physics.gravity_air;
        } else {
            self.ym += physics.gravity_ground;
        }

        if !state.flags.control_enabled() || !state.key_trigger.jump() {
            if self.flags.flag_x10() && self.xm < 0 {
                self.ym = -self.xm;
            }

            if self.flags.flag_x20() && self.xm > 0 {
                self.ym = self.xm;
            }

            if (self.flags.flag_x08() && self.flags.flag_x80000() && self.xm < 0)
                || (self.flags.flag_x08() && self.flags.flag_x10000() && self.xm > 0)
                || (self.flags.flag_x08() && self.flags.flag_x20000() && self.flags.flag_x40000()) {
                self.ym = 0x400; // 2.0fix9
            }
        }

        let max_move = if self.flags.underwater() && !(self.flags.force_left() || self.flags.force_up() || self.flags.force_right() || self.flags.force_down()) {
            self.water_physics.max_move
        } else {
            self.air_physics.max_move
        };

        self.xm = clamp(self.xm, -max_move, max_move);
        self.ym = clamp(self.ym, -max_move, max_move);

        // todo: water splashing

        if !self.flags.underwater() {
            self.sprash = false;
        }

        // spike damage
        if self.flags.flag_x400() {
            //self.damage(10); // todo: borrow checker yells at me
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

        if state.flags.control_enabled() && state.key_state.up() {
            self.index_x -= 0x200; // 1.0fix9
            if self.index_x < -0x8000 { // -64.0fix9
                self.index_x = -0x8000;
            }
        } else if state.flags.control_enabled() && state.key_state.down() {
            self.index_x += 0x200; // 1.0fix9
            if self.index_x > 0x8000 { // -64.0fix9
                self.index_x = 0x8000;
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

        if self.xm > physics.resist || self.xm < -physics.resist {
            self.x += self.xm;
        }

        self.y += self.ym;

        Ok(())
    }

    fn tick_stream(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn tick_animation(&mut self, state: &SharedGameState) {
        if self.cond.cond_x02() {
            return;
        }

        if self.flags.flag_x08() {
            if self.cond.cond_x01() {
                self.anim_num = 11;
            } else if state.flags.control_enabled() && state.key_state.up() && (state.key_state.left() || state.key_state.right()) {
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
            } else if state.flags.control_enabled() && (state.key_state.left() || state.key_state.right()) {
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
            } else if state.flags.control_enabled() && state.key_state.up() {
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
            self.anim_num = if self.ym > 0 { 1 } else { 3 };
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

    pub fn damage(&mut self, hp: isize) {}
}

impl GameEntity for Player {
    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult<()> {
        if !self.cond.visible() {
            return Ok(());
        }

        if self.exp_wait != 0 {
            self.exp_wait -= 1;
        }

        if self.shock != 0 {
            self.shock -= 1;
        } else if self.exp_count != 0 {
            // SetValueView(&self.x, &self.y, self.exp_count); // todo: damage popup
            self.exp_count = 0;
        }

        match self.unit {
            0 => {
                if state.flags.flag_x04() && state.flags.control_enabled() {
                    // AirProcess(); // todo
                }

                self.tick_normal(state, ctx)?;
            }
            1 => {
                self.tick_stream(state, ctx)?;
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

        let sb = state.texture_set.tex_map.get_mut(&self.tex_player_name);
        if sb.is_none() {
            return Ok(());
        }

        let batch = sb.unwrap();
        batch.add_rect(
            (((self.x - self.view.left as isize) / 0x200) - (frame.x / 0x200)) as f32,
            (((self.y - self.view.top as isize) / 0x200) - (frame.y / 0x200)) as f32,
            &self.anim_rect,
        );
        batch.draw(ctx)?;

        Ok(())
    }
}
