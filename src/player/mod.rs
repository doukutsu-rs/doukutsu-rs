use std::clone::Clone;

use num_derive::FromPrimitive;
use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{interpolate_fix9_scale, Condition, Direction, Equipment, Flag, Rect};
use crate::components::number_popup::NumberPopup;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::dummy_player_controller::DummyPlayerController;
use crate::input::player_controller::PlayerController;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::skin::basic::BasicPlayerSkin;
use crate::player::skin::{PlayerAnimationState, PlayerAppearanceState, PlayerSkin};
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

mod player_hit;
pub mod skin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum ControlMode {
    Normal = 0,
    IronHead,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TargetPlayer {
    Player1,
    Player2,
}

impl TargetPlayer {
    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum BoosterSwitch {
    None,
    Up,
    Left,
    Right,
    Down,
}

#[derive(Clone)]
struct DogStack {
    pub offset_x: f32,
    pub speed: f32,
    pub prev_speed: f32,
}

impl DogStack {
    const TENSION: f32 = 0.25;
    const DAMPENING: f32 = 0.1;
    const MULT: f32 = 1.2;

    pub fn new() -> DogStack {
        DogStack { offset_x: 0.0, speed: 0.0, prev_speed: 0.0 }
    }

    pub fn tick(&mut self) {
        self.prev_speed = self.speed;
        self.speed += -self.offset_x * DogStack::TENSION as f32 - self.speed * DogStack::DAMPENING;
        self.offset_x += self.speed;
    }
}

#[derive(Clone)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub camera_target_x: i32,
    pub camera_target_y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub life: u16,
    pub max_life: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub equip: Equipment,
    pub direction: Direction,
    pub display_bounds: Rect<u32>,
    pub hit_bounds: Rect<u32>,
    pub control_mode: ControlMode,
    pub physical: bool,
    pub question: bool,
    pub booster_fuel: u32,
    pub up: bool,
    pub down: bool,
    pub shock_counter: u8,
    pub xp_counter: u8,
    pub current_weapon: u8,
    pub stars: u8,
    pub damage: u16,
    pub air_counter: u16,
    pub air: u16,
    pub skin: Box<dyn PlayerSkin>,
    pub controller: Box<dyn PlayerController>,
    pub popup: NumberPopup,
    strafe_up: bool,
    weapon_offset_y: i8,
    splash: bool,
    tick: u8,
    booster_switch: BoosterSwitch,
    damage_counter: u16,
    damage_taken: i16,
    pub anim_num: u16,
    anim_counter: u16,
    anim_rect: Rect<u16>,
    weapon_rect: Rect<u16>,
    dog_stack: Vec<DogStack>,
    pub has_dog: bool,
}

impl Player {
    pub fn new(state: &mut SharedGameState, ctx: &mut Context) -> Player {
        let constants = &state.constants;
        let skin = Box::new(BasicPlayerSkin::new("MyChar".to_string(), state, ctx));

        Player {
            x: 0,
            y: 0,
            vel_x: 0,
            vel_y: 0,
            target_x: 0,
            target_y: 0,
            camera_target_x: 0,
            camera_target_y: 0,
            prev_x: 0,
            prev_y: 0,
            life: constants.player.life,
            max_life: constants.player.max_life,
            cond: Condition(0),
            flags: Flag(0),
            equip: Equipment(0),
            direction: Direction::Right,
            display_bounds: skin.get_display_bounds(),
            hit_bounds: skin.get_hit_bounds(),
            control_mode: constants.player.control_mode,
            physical: true,
            question: false,
            booster_fuel: 0,
            splash: false,
            up: false,
            down: false,
            current_weapon: 0,
            weapon_offset_y: 0,
            shock_counter: 0,
            xp_counter: 0,
            tick: 0,
            booster_switch: BoosterSwitch::None,
            stars: 0,
            damage: 0,
            air_counter: 0,
            air: 0,
            skin,
            controller: Box::new(DummyPlayerController::new()),
            popup: NumberPopup::new(),
            strafe_up: false,
            damage_counter: 0,
            damage_taken: 0,
            anim_num: 0,
            anim_counter: 0,
            anim_rect: constants.player.frames_right[0],
            weapon_rect: Rect::new(0, 0, 0, 0),
            dog_stack: Vec::new(),
            has_dog: false,
        }
    }

    pub fn get_texture_offset(&self) -> u16 {
        if self.equip.has_mimiga_mask() {
            32
        } else {
            0
        }
    }

    fn tick_normal(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if !state.control_flags.interactions_disabled() && state.control_flags.control_enabled() {
            if self.equip.has_air_tank() {
                self.air = 1000;
                self.air_counter = 0;
            } else if !state.settings.god_mode && self.flags.in_water() {
                self.air_counter = 60;
                if self.air > 0 {
                    self.air -= 1;
                } else if state.get_flag(4000) {
                    state.textscript_vm.start_script(1100);
                } else {
                    // Switch uses player sprite for drowned effect
                    if !state.constants.is_switch {
                        self.cond.set_hidden(true);
                        state.create_caret(self.x, self.y, CaretType::DrownedQuote, self.direction);
                    }
                    state.textscript_vm.start_script(41);
                }
            } else {
                self.air = 1000;

                if self.air_counter > 0 {
                    self.air_counter -= 1;
                }
            }
        }

        if self.cond.hidden() {
            return Ok(());
        }

        let physics = if self.flags.in_water() {
            state.constants.player.water_physics
        } else {
            state.constants.player.air_physics
        };

        self.question = false;

        if !state.control_flags.control_enabled() {
            self.booster_switch = BoosterSwitch::None;
        }

        if state.control_flags.control_enabled() {
            if self.controller.trigger_strafe() {
                if self.controller.move_up() {
                    self.strafe_up = true;
                }
            } else if !self.controller.strafe() {
                self.strafe_up = false;
            }
        } else {
            self.strafe_up = false;
        }

        // ground movement
        if self.flags.hit_bottom_wall() || self.flags.hit_right_slope() || self.flags.hit_left_slope() {
            self.booster_switch = BoosterSwitch::None;

            if state.settings.infinite_booster {
                self.booster_fuel = u32::MAX;
            } else if self.equip.has_booster_0_8() || self.equip.has_booster_2_0() {
                self.booster_fuel = state.constants.booster.fuel;
            } else {
                self.booster_fuel = 0;
            }

            if state.control_flags.control_enabled() {
                let trigger_only_down = self.controller.trigger_down()
                    && !self.controller.trigger_up()
                    && !self.controller.trigger_left()
                    && !self.controller.trigger_right()
                    && !self.controller.trigger_shoot();

                let only_down = self.controller.move_down()
                    && !self.controller.move_up()
                    && !self.controller.move_left()
                    && !self.controller.move_right()
                    && !self.controller.shoot();

                if trigger_only_down
                    && only_down
                    && !self.cond.interacted()
                    && !state.control_flags.interactions_disabled()
                {
                    self.cond.set_interacted(true);
                    self.question = true;
                } else {
                    if self.controller.move_left() && self.vel_x > -physics.max_dash {
                        self.vel_x -= physics.dash_ground;
                    }

                    if self.controller.move_right() && self.vel_x < physics.max_dash {
                        self.vel_x += physics.dash_ground;
                    }

                    if !self.controller.strafe() {
                        if self.controller.move_left() {
                            self.direction = Direction::Left;
                        }

                        if self.controller.move_right() {
                            self.direction = Direction::Right;
                        }
                    }
                }
            }

            if !self.cond.increase_acceleration() {
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
        } else {
            // air movement
            if state.control_flags.control_enabled() {
                if self.controller.trigger_jump() && self.booster_fuel != 0 {
                    if self.equip.has_booster_0_8() {
                        self.booster_switch = BoosterSwitch::Up;

                        if self.vel_y > 0x100 {
                            self.vel_y /= 2;
                        }
                    } else if state.settings.infinite_booster || self.equip.has_booster_2_0() {
                        if self.controller.move_up() {
                            self.booster_switch = BoosterSwitch::Up;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_up;
                        } else if self.controller.move_left() {
                            self.booster_switch = BoosterSwitch::Left;
                            self.vel_x = state.constants.booster.b2_0_left;
                            self.vel_y = 0;
                        } else if self.controller.move_right() {
                            self.booster_switch = BoosterSwitch::Right;
                            self.vel_x = state.constants.booster.b2_0_right;
                            self.vel_y = 0;
                        } else if self.controller.move_down() {
                            self.booster_switch = BoosterSwitch::Down;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_down;
                        } else {
                            self.booster_switch = BoosterSwitch::Up;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_up_nokey;
                        }
                    }
                }

                if self.controller.move_left() && self.vel_x > -physics.max_dash {
                    self.vel_x -= physics.dash_air;
                }

                if self.controller.move_right() && self.vel_x < physics.max_dash {
                    self.vel_x += physics.dash_air;
                }

                if !self.controller.strafe() {
                    if self.controller.look_left() {
                        self.direction = Direction::Left;
                    }

                    if self.controller.look_right() {
                        self.direction = Direction::Right;
                    }
                }
            }

            if (state.settings.infinite_booster || self.equip.has_booster_2_0())
                && self.booster_switch != BoosterSwitch::None
                && (!self.controller.jump() || self.booster_fuel == 0)
            {
                match self.booster_switch {
                    BoosterSwitch::Left | BoosterSwitch::Right => self.vel_x /= 2,
                    BoosterSwitch::Up => self.vel_y /= 2,
                    _ => (),
                }
            }

            if self.booster_fuel == 0 || !self.controller.jump() {
                self.booster_switch = BoosterSwitch::None;
            }
        }

        // jumping
        if state.control_flags.control_enabled() {
            self.up = self.controller.move_up() || self.strafe_up;
            self.down = self.controller.move_down() && !self.flags.hit_bottom_wall();

            if self.controller.trigger_jump()
                && (self.flags.hit_bottom_wall() || self.flags.hit_right_slope() || self.flags.hit_left_slope())
                && !self.flags.force_up()
            {
                self.vel_y = -physics.jump;
                state.sound_manager.play_sfx(15);
            }
        }

        // stop interacting when moved
        if state.control_flags.control_enabled()
            && (self.controller.move_left()
                || self.controller.move_right()
                || self.controller.move_up()
                || self.controller.jump()
                || self.controller.shoot())
        {
            self.cond.set_interacted(false);
        }

        // booster losing fuel
        if self.booster_switch != BoosterSwitch::None && self.booster_fuel != 0 {
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
            self.vel_x += 0x88;
        }
        if self.flags.force_down() {
            self.vel_y += 0x55;
        }

        if (state.settings.infinite_booster || self.equip.has_booster_2_0())
            && self.booster_switch != BoosterSwitch::None
        {
            match self.booster_switch {
                BoosterSwitch::Left | BoosterSwitch::Right => {
                    if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                        self.vel_y = -0x100;
                    }

                    let mut booster_dir = self.direction;

                    if self.controller.strafe() {
                        if self.controller.move_left() {
                            self.booster_switch = BoosterSwitch::Left;
                        } else if self.controller.move_right() {
                            self.booster_switch = BoosterSwitch::Right;
                        }

                        if self.booster_switch == BoosterSwitch::Left {
                            booster_dir = Direction::Left;
                        } else if self.booster_switch == BoosterSwitch::Right {
                            booster_dir = Direction::Right;
                        }
                    }

                    self.vel_x += match booster_dir {
                        Direction::Left => -0x20,
                        Direction::Right => 0x20,
                        _ => 0,
                    };

                    if self.controller.trigger_jump() || self.booster_fuel % 3 == 1 {
                        if self.direction == Direction::Left || self.direction == Direction::Right {
                            state.create_caret(
                                self.x + 0x400,
                                self.y + 0x400,
                                CaretType::Exhaust,
                                booster_dir.opposite(),
                            );
                        }
                        state.sound_manager.play_sfx(113);
                    }
                }
                BoosterSwitch::Up => {
                    self.vel_y -= 0x20;

                    if self.controller.trigger_jump() || self.booster_fuel % 3 == 1 {
                        state.create_caret(self.x, self.y + 0xc00, CaretType::Exhaust, Direction::Bottom);
                        state.sound_manager.play_sfx(113);
                    }
                }
                BoosterSwitch::Down if self.controller.trigger_jump() || self.booster_fuel % 3 == 1 => {
                    state.create_caret(self.x, self.y - 0xc00, CaretType::Exhaust, Direction::Up);
                    state.sound_manager.play_sfx(113);
                }
                _ => {}
            }
        } else if self.flags.force_up() {
            self.vel_y += physics.gravity_ground;
        } else if self.equip.has_booster_0_8() && self.booster_switch != BoosterSwitch::None && self.vel_y > -0x400 {
            self.vel_y -= 0x20;

            if self.booster_fuel % 3 == 0 {
                state.create_caret(
                    self.x,
                    self.y + self.hit_bounds.bottom as i32 / 2,
                    CaretType::Exhaust,
                    Direction::Bottom,
                );
                state.sound_manager.play_sfx(113);
            }

            // bounce off of ceiling
            if self.flags.hit_top_wall() {
                self.vel_y = 0x200; // 1.0fix9
            }
        } else if self.vel_y < 0 && state.control_flags.control_enabled() && self.controller.jump() {
            self.vel_y += physics.gravity_air;
        } else {
            self.vel_y += physics.gravity_ground;
        }

        if !state.control_flags.control_enabled() || !self.controller.trigger_jump() {
            if self.flags.hit_right_slope() && self.vel_x < 0 {
                self.vel_y = -self.vel_x;
            }

            if self.flags.hit_left_slope() && self.vel_x > 0 {
                self.vel_y = self.vel_x;
            }

            if (self.flags.hit_bottom_wall() && self.flags.hit_right_higher_half() && self.vel_x < 0)
                || (self.flags.hit_bottom_wall() && self.flags.hit_left_higher_half() && self.vel_x > 0)
                || (self.flags.hit_bottom_wall()
                    && self.flags.hit_left_lower_half()
                    && self.flags.hit_right_lower_half())
            {
                self.vel_y = 0x400; // 2.0fix9
            }
        }

        let max_move = if self.flags.in_water()
            && !(self.flags.force_left()
                || self.flags.force_up()
                || self.flags.force_right()
                || self.flags.force_down())
        {
            state.constants.player.water_physics.max_move
        } else {
            state.constants.player.air_physics.max_move
        };

        self.vel_x = self.vel_x.clamp(-max_move, max_move);
        self.vel_y = self.vel_y.clamp(-max_move, max_move);

        if !self.splash && self.flags.in_water() {
            let vertical_splash = !self.flags.hit_bottom_wall() && self.vel_y > 0x200;
            let horizontal_splash = self.vel_x > 0x200 || self.vel_x < -0x200;

            if vertical_splash || horizontal_splash {
                let mut droplet = NPC::create(73, &state.npc_table);
                droplet.cond.set_alive(true);
                droplet.y = self.y;
                droplet.direction =
                    if self.flags.water_splash_facing_right() { Direction::Right } else { Direction::Left };

                for _ in 0..7 {
                    droplet.x = self.x + (state.game_rng.range(-8..8) * 0x200) as i32;

                    droplet.vel_x = self.vel_x + state.game_rng.range(-0x200..0x200);
                    droplet.vel_y = match () {
                        _ if vertical_splash => state.game_rng.range(-0x200..0x80) - (self.vel_y / 2),
                        _ if horizontal_splash => state.game_rng.range(-0x200..0x80),
                        _ => 0,
                    };

                    let _ = npc_list.spawn(0x100, droplet.clone());
                }

                state.sound_manager.play_sfx(56);
            }

            self.splash = true;
        }

        if !self.flags.in_water() {
            self.splash = false;
        }

        // spike damage
        if self.flags.hit_by_spike() {
            self.damage(10, state, npc_list);
        }

        // camera
        self.camera_target_x = clamp(self.camera_target_x + self.direction.vector_x() * 0x200, -0x8000, 0x8000);

        if state.control_flags.control_enabled() && self.controller.look_up() {
            self.camera_target_y -= 0x200;
            if self.camera_target_y < -0x8000 {
                // -64.0fix9
                self.camera_target_y = -0x8000;
            }
        } else if state.control_flags.control_enabled() && self.controller.look_down() {
            self.camera_target_y += 0x200;
            if self.camera_target_y > 0x8000 {
                // -64.0fix9
                self.camera_target_y = 0x8000;
            }
        } else {
            if self.camera_target_y > 0x200 {
                self.camera_target_y -= 0x200;
            }

            if self.camera_target_y < -0x200 {
                self.camera_target_y += 0x200;
            }
        }

        self.target_x = self.x + self.camera_target_x;
        self.target_y = self.y + self.camera_target_y;

        if self.vel_x > physics.resist || self.vel_x < -physics.resist {
            self.x += self.vel_x;
        }

        self.y += self.vel_y;

        Ok(())
    }

    fn tick_ironhead(&mut self, state: &mut SharedGameState) -> GameResult {
        self.up = false;
        self.down = false;
        if state.control_flags.control_enabled() {
            if self.controller.move_left() || self.controller.move_right() {
                if self.controller.move_left() {
                    self.vel_x -= 0x100;
                }
                if self.controller.move_right() {
                    self.vel_x += 0x100;
                }
            } else if self.vel_x > 0x7f || self.vel_x < -0x7f {
                self.vel_x += 0x80 * -self.vel_x.signum();
            } else {
                self.vel_x = 0;
            }

            if self.controller.move_up() || self.controller.move_down() {
                if self.controller.move_up() {
                    self.vel_y -= 0x100;
                }
                if self.controller.move_down() {
                    self.vel_y += 0x100;
                }
            } else if self.vel_y > 0x7f || self.vel_y < -0x7f {
                self.vel_y += 0x80 * -self.vel_y.signum();
            } else {
                self.vel_y = 0;
            }
            if state.settings.noclip {
                self.target_x = self.x + self.camera_target_x;
                self.target_y = self.y + self.camera_target_y;
            }
        } else {
            if self.vel_x > 0x7f || self.vel_x < -0x7f {
                self.vel_x += 0x80 * -self.vel_x.signum();
            } else {
                self.vel_x = 0;
            }

            if self.vel_y > 0x7f || self.vel_y < -0x7f {
                self.vel_y += 0x80 * -self.vel_y.signum();
            } else {
                self.vel_y = 0;
            }
        }
        //Toggles bonk particles
        if state.settings.noclip == false {
            if self.vel_y < -0x200 && self.flags.hit_top_wall() {
                state.create_caret(
                    self.x,
                    self.y - self.hit_bounds.top as i32,
                    CaretType::LittleParticles,
                    Direction::FacingPlayer,
                );
            }

            if self.vel_y > 0x200 && self.flags.hit_bottom_wall() {
                state.create_caret(
                    self.x,
                    self.y + self.hit_bounds.bottom as i32,
                    CaretType::LittleParticles,
                    Direction::FacingPlayer,
                );
            }
        }
        self.vel_x = self.vel_x.clamp(-0x400, 0x400);
        self.vel_y = self.vel_y.clamp(-0x400, 0x400);

        if self.controller.move_left() && self.controller.move_up() {
            if self.vel_x < -0x30c {
                self.vel_x = -0x30c;
            }
            if self.vel_y < -0x30c {
                self.vel_y = -0x30c;
            }
        }

        if self.controller.move_right() && self.controller.move_up() {
            if self.vel_x > 0x30c {
                self.vel_x = 0x30c;
            }
            if self.vel_y < -0x30c {
                self.vel_y = -0x30c;
            }
        }

        if self.controller.move_left() && self.controller.move_down() {
            if self.vel_x < -0x30c {
                self.vel_x = -0x30c;
            }
            if self.vel_y > 0x30c {
                self.vel_y = 0x30c;
            }
        }

        if self.controller.move_right() && self.controller.move_down() {
            if self.vel_x > 0x30c {
                self.vel_x = 0x30c;
            }
            if self.vel_y > 0x30c {
                self.vel_y = 0x30c;
            }
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    fn tick_animation(&mut self, state: &mut SharedGameState) {
        if self.cond.hidden() {
            return;
        }

        if self.flags.hit_bottom_wall() {
            if self.cond.interacted() {
                self.skin.set_state(PlayerAnimationState::Examining);
                self.anim_num = 0;
                self.anim_counter = 0;
            } else if state.control_flags.control_enabled()
                && (self.controller.move_up() || self.strafe_up)
                && (self.controller.move_left() || self.controller.move_right())
            {
                self.cond.set_fallen(true);
                self.skin.set_state(PlayerAnimationState::WalkingUp);

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
            } else if state.control_flags.control_enabled()
                && (self.controller.move_left() || self.controller.move_right())
            {
                self.cond.set_fallen(true);
                self.skin.set_state(PlayerAnimationState::Walking);

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
            } else if state.control_flags.control_enabled() && (self.controller.move_up() || self.strafe_up) {
                if self.cond.fallen() {
                    state.sound_manager.play_sfx(24);
                }

                self.cond.set_fallen(false);
                self.skin.set_state(PlayerAnimationState::LookingUp);
                self.anim_num = 0;
                self.anim_counter = 0;
            } else {
                if self.cond.fallen() {
                    state.sound_manager.play_sfx(24);
                }

                self.cond.set_fallen(false);
                self.skin.set_state(PlayerAnimationState::Idle);
                self.anim_num = 0;
                self.anim_counter = 0;
            }
        } else if (self.controller.look_up() || self.strafe_up) && self.control_mode == ControlMode::Normal {
            self.skin.set_state(PlayerAnimationState::FallingLookingUp);
            self.anim_num = 0;
            self.anim_counter = 0;
        } else if self.controller.look_down() && self.control_mode == ControlMode::Normal {
            self.skin.set_state(PlayerAnimationState::FallingLookingDown);
            self.anim_num = 0;
            self.anim_counter = 0;
        } else {
            self.skin.set_state(if self.vel_y > 0 {
                PlayerAnimationState::Jumping
            } else {
                PlayerAnimationState::Falling
            });
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        self.weapon_offset_y = 0;
        self.weapon_rect.left = (self.current_weapon as u16 % 13) * 24;
        self.weapon_rect.top = (self.current_weapon as u16 / 13) * 96;
        self.weapon_rect.right = self.weapon_rect.left + 24;
        self.weapon_rect.bottom = self.weapon_rect.top + 16;

        if self.direction == Direction::Right {
            self.weapon_rect.top += 16;
            self.weapon_rect.bottom += 16;
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

        self.skin.tick();
        self.skin.set_direction(self.direction);
        self.skin.set_appearance(if self.equip.has_mimiga_mask() {
            PlayerAppearanceState::MimigaMask
        } else {
            PlayerAppearanceState::Default
        });

        if state.constants.is_switch && self.air == 0 && self.flags.in_water() && !state.get_flag(4000) {
            self.skin.set_appearance(PlayerAppearanceState::Default);
            self.skin.set_state(PlayerAnimationState::Drowned);
        }

        self.anim_rect = self.skin.animation_frame();

        self.tick = self.tick.wrapping_add(1);
    }

    pub fn damage(&mut self, hp: i32, state: &mut SharedGameState, npc_list: &NPCList) {
        if self.life == 0 || hp <= 0 || state.settings.god_mode || self.shock_counter > 0 {
            return;
        }

        state.sound_manager.play_sfx(16);
        self.shock_counter = 128;
        self.cond.set_interacted(false);

        if self.control_mode == ControlMode::Normal {
            self.vel_y = -0x400; // -2.0fix9
        }

        let final_hp = state.get_damage(hp);

        self.life = self.life.saturating_sub(final_hp as u16);

        if self.equip.has_whimsical_star() && self.stars > 0 {
            self.stars -= 1;
        }

        self.damage = self.damage.saturating_add(final_hp as u16);
        if self.popup.value > 0 {
            self.popup.set_value(-(self.damage as i16));
        } else {
            self.popup.add_value(-(self.damage as i16));
        }

        if self.life == 0 {
            state.sound_manager.play_sfx(17);
            self.cond.0 = 0;
            state.control_flags.set_tick_world(true);
            state.control_flags.set_interactions_disabled(true);
            state.textscript_vm.start_script(40);

            state.create_caret(self.x, self.y, CaretType::Explosion, Direction::Left);
            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            for _ in 0..0x40 {
                npc.x = self.x + state.game_rng.range(-10..10) as i32 * 0x200;
                npc.y = self.y + state.game_rng.range(-10..10) as i32 * 0x200;

                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }
    }
}

impl GameEntity<&NPCList> for Player {
    fn tick(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if !self.cond.alive() {
            return Ok(());
        }

        if state.textscript_vm.reset_invicibility && state.constants.textscript.reset_invicibility_on_any_script {
            self.shock_counter = 0;
        }

        if self.damage_counter != 0 {
            self.damage_counter -= 1;
        }

        if self.xp_counter != 0 {
            self.xp_counter -= 1;
        }

        if self.shock_counter != 0 {
            self.shock_counter -= 1;
        } else if self.damage_taken != 0 {
            self.damage_taken = 0;
        }

        match (self.control_mode, state.settings.noclip) {
            (_, true) => self.tick_ironhead(state)?,
            (ControlMode::Normal, _) => self.tick_normal(state, npc_list)?,
            (ControlMode::IronHead, _) => self.tick_ironhead(state)?,
        }

        self.popup.x = self.x;
        self.popup.y = self.y - self.display_bounds.top as i32 + 0x1000;
        self.popup.tick(state, ())?;

        self.cond.set_increase_acceleration(false);
        self.tick_animation(state);

        let dog_amount = (3000..=3005).filter(|id| state.get_flag(*id as usize)).count();
        self.dog_stack.resize(dog_amount, DogStack::new());

        if self.has_dog && self.dog_stack.len() > 0 {
            if self.vel_x.abs() > 0x100 {
                self.dog_stack[0].speed = (self.vel_x / 2) as f32;
            }
            for i in 1..self.dog_stack.len() {
                self.dog_stack[i].speed = self.dog_stack[i - 1].prev_speed * DogStack::MULT;
            }
            for dog in &mut self.dog_stack {
                dog.tick();
            }
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        if !self.cond.alive() || self.cond.hidden() {
            return Ok(());
        }

        let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

        // hack for stacked dogs
        if self.has_dog {
            if self.dog_stack.len() > 0 {
                let vec_x = self.direction.vector_x() * 0x800;
                let vec_y = 0x1400;

                if let Some(entry) = state.npc_table.get_entry(136) {
                    let sprite = &*state.npc_table.get_texture_ref(entry.spritesheet_id as u16);
                    let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, sprite)?;

                    let (off_x, frame_id) = if self.direction == Direction::Left {
                        (entry.display_bounds.right as i32 * 0x200, 0)
                    } else {
                        (entry.display_bounds.left as i32 * 0x200, 2)
                    };
                    let off_y = entry.display_bounds.top as i32 * 0x200;

                    for i in (1..=(self.dog_stack.len() as i32)).rev() {
                        batch.add_rect(
                            interpolate_fix9_scale(
                                self.prev_x - off_x - vec_x * i - self.dog_stack[i as usize - 1].offset_x as i32,
                                self.x - off_x - vec_x * i - self.dog_stack[i as usize - 1].offset_x as i32,
                                state.frame_time,
                            ) - frame_x,
                            interpolate_fix9_scale(
                                self.prev_y - off_y - vec_y * i - (self.y - self.prev_y) * i,
                                self.y - off_y - vec_y * i - (self.y - self.prev_y) * i,
                                state.frame_time,
                            ) - frame_y,
                            &state.constants.npc.n136_puppy_carried[frame_id],
                        );
                    }

                    batch.draw(ctx)?;
                }
            }
        }

        if self.shock_counter / 2 % 2 != 0 {
            return Ok(());
        }

        if self.current_weapon != 0 {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Arms")?;
            let (gun_off_x, gun_off_y) = self.skin.get_gun_offset();

            batch.add_rect(
                interpolate_fix9_scale(
                    self.prev_x - self.display_bounds.left as i32,
                    self.x - self.display_bounds.left as i32,
                    state.frame_time,
                ) + if self.direction == Direction::Left { -8.0 - gun_off_x as f32 } else { gun_off_x as f32 }
                    - frame_x,
                interpolate_fix9_scale(
                    self.prev_y - self.display_bounds.top as i32,
                    self.y - self.display_bounds.top as i32,
                    state.frame_time,
                ) + self.weapon_offset_y as f32
                    + gun_off_y as f32
                    - frame_y,
                &self.weapon_rect,
            );

            batch.draw(ctx)?;
        }

        {
            let batch =
                state.texture_set.get_or_load_batch(ctx, &state.constants, self.skin.get_skin_texture_name())?;
            batch.add_rect(
                interpolate_fix9_scale(
                    self.prev_x - self.display_bounds.left as i32,
                    self.x - self.display_bounds.left as i32,
                    state.frame_time,
                ) - frame_x,
                interpolate_fix9_scale(
                    self.prev_y - self.display_bounds.top as i32,
                    self.y - self.display_bounds.top as i32,
                    state.frame_time,
                ) - frame_y,
                &self.anim_rect,
            );
            batch.draw(ctx)?;
        }

        if (self.equip.has_air_tank() && self.flags.in_water()) || self.control_mode == ControlMode::IronHead {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Caret")?;
            batch.add_rect(
                interpolate_fix9_scale(self.prev_x - 12 * 0x200, self.x - 12 * 0x200, state.frame_time) - frame_x,
                interpolate_fix9_scale(self.prev_y - 12 * 0x200, self.y - 12 * 0x200, state.frame_time) - frame_y,
                &state.constants.player.frames_bubble[(self.tick / 2 % 2) as usize],
            );
            batch.draw(ctx)?;
        }

        Ok(())
    }
}
