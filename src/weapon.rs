use num_derive::FromPrimitive;

use crate::bullet::{Bullet, BulletManager};
use crate::caret::CaretType;
use crate::common::Direction;
use crate::inventory::Inventory;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;

#[derive(Debug, PartialEq, Eq, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum WeaponType {
    None = 0,
    Snake = 1,
    PolarStar = 2,
    Fireball = 3,
    MachineGun = 4,
    MissileLauncher = 5,
    Bubbler = 7,
    Blade = 9,
    SuperMissileLauncher = 10,
    Nemesis = 12,
    Spur = 13,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum WeaponLevel {
    None = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}

impl WeaponLevel {
    pub fn next(self) -> WeaponLevel {
        match self {
            WeaponLevel::None => WeaponLevel::Level1,
            WeaponLevel::Level1 => WeaponLevel::Level2,
            WeaponLevel::Level2 => WeaponLevel::Level3,
            WeaponLevel::Level3 => WeaponLevel::Level3,
        }
    }

    pub fn prev(self) -> WeaponLevel {
        match self {
            WeaponLevel::None => WeaponLevel::Level1,
            WeaponLevel::Level1 => WeaponLevel::Level1,
            WeaponLevel::Level2 => WeaponLevel::Level1,
            WeaponLevel::Level3 => WeaponLevel::Level2,
        }
    }
}

#[derive(Clone)]
pub struct Weapon {
    pub wtype: WeaponType,
    pub level: WeaponLevel,
    pub experience: u16,
    pub ammo: u16,
    pub max_ammo: u16,
    counter1: u16,
    counter2: u16,
}

impl Weapon {
    pub fn new(wtype: WeaponType, level: WeaponLevel, experience: u16, ammo: u16, max_ammo: u16) -> Weapon {
        Weapon { wtype, level, experience, ammo, max_ammo, counter1: 0, counter2: 0 }
    }

    /// Consume a specified amount of bullets, returns true if there was enough ammo.
    pub fn consume_ammo(&mut self, amount: u16) -> bool {
        if self.max_ammo == 0 {
            return true;
        }

        if self.ammo >= amount {
            self.ammo -= amount;
            return true;
        }

        false
    }

    /// Refill a specified amount of bullets.
    pub fn refill_ammo(&mut self, amount: u16) {
        if self.max_ammo != 0 {
            self.ammo = self.ammo.saturating_add(amount).min(self.max_ammo);
        }
    }

    fn tick_snake(&mut self, player: &Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        if player.controller.trigger_shoot() && bullet_manager.count_bullets_multi(&[1, 2, 3], player_id) < 4 {
            let btype = match self.level {
                WeaponLevel::Level1 => 1,
                WeaponLevel::Level2 => 2,
                WeaponLevel::Level3 => 3,
                WeaponLevel::None => {
                    unreachable!()
                }
            };

            if !self.consume_ammo(1) {
                // todo switch to first weapon
                return;
            }

            self.counter1 = self.counter1.wrapping_add(1);

            if player.up {
                match player.direction {
                    Direction::Left => {
                        let mut bullet = Bullet::new(player.x - 0x600, player.y - 10 * 0x200, btype, player_id, Direction::Up, &state.constants);
                        bullet.target_x = self.counter1 as i32;
                        bullet_manager.push_bullet(bullet);
                        state.create_caret(player.x - 0x600, player.y - 10 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        let mut bullet = Bullet::new(player.x + 0x600, player.y - 10 * 0x200, btype, player_id, Direction::Up, &state.constants);
                        bullet.target_x = self.counter1 as i32;
                        bullet_manager.push_bullet(bullet);
                        state.create_caret(player.x + 0x600, player.y - 10 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else if player.down {
                match player.direction {
                    Direction::Left => {
                        let mut bullet = Bullet::new(player.x - 0x600, player.y + 10 * 0x200, btype, player_id, Direction::Bottom, &state.constants);
                        bullet.target_x = self.counter1 as i32;
                        bullet_manager.push_bullet(bullet);
                        state.create_caret(player.x - 0x600, player.y + 10 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        let mut bullet = Bullet::new(player.x + 0x600, player.y + 10 * 0x200, btype, player_id, Direction::Bottom, &state.constants);
                        bullet.target_x = self.counter1 as i32;
                        bullet_manager.push_bullet(bullet);
                        state.create_caret(player.x + 0x600, player.y + 10 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else {
                match player.direction {
                    Direction::Left => {
                        let mut bullet = Bullet::new(player.x - 0xc00, player.y + 0x400, btype, player_id, Direction::Left, &state.constants);
                        bullet.target_x = self.counter1 as i32;
                        bullet_manager.push_bullet(bullet);
                        state.create_caret(player.x - 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        let mut bullet = Bullet::new(player.x + 0xc00, player.y + 0x400, btype, player_id, Direction::Right, &state.constants);
                        bullet.target_x = self.counter1 as i32;
                        bullet_manager.push_bullet(bullet);
                        state.create_caret(player.x + 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Right);
                    }
                    _ => {}
                }
            }

            state.sound_manager.play_sfx(33);
        }
    }

    fn tick_polar_star(&mut self, player: &Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        if player.controller.trigger_shoot() && bullet_manager.count_bullets_multi(&[4, 5, 6], player_id) < 2 {
            let btype = match self.level {
                WeaponLevel::Level1 => 4,
                WeaponLevel::Level2 => 5,
                WeaponLevel::Level3 => 6,
                WeaponLevel::None => {
                    unreachable!()
                }
            };

            if !self.consume_ammo(1) {
                state.sound_manager.play_sfx(37);
                return;
            }

            if player.up {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x - 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x + 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else if player.down {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x - 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x + 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0xc00, player.y + 0x600, btype, player_id, Direction::Left, &state.constants);
                        state.create_caret(player.x - 0xc00, player.y + 0x600, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0xc00, player.y + 0x600, btype, player_id, Direction::Right, &state.constants);
                        state.create_caret(player.x + 0xc00, player.y + 0x600, CaretType::Shoot, Direction::Right);
                    }
                    _ => {}
                }
            }

            if self.level == WeaponLevel::Level3 {
                state.sound_manager.play_sfx(49);
            } else {
                state.sound_manager.play_sfx(32);
            }
        }
    }

    fn tick_fireball(&mut self, player: &Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        let max_bullets = self.level as usize + 1;
        if player.controller.trigger_shoot() && bullet_manager.count_bullets_multi(&[7, 8, 9], player_id) < max_bullets {
            let btype = match self.level {
                WeaponLevel::Level1 => 7,
                WeaponLevel::Level2 => 8,
                WeaponLevel::Level3 => 9,
                WeaponLevel::None => {
                    unreachable!()
                }
            };

            if !self.consume_ammo(1) {
                // todo switch to first weapon
                return;
            }

            if player.up {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x800, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x - 0x800, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x800, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x + 0x800, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else if player.down {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x800, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x - 0x800, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x800, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x + 0x800, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0xc00, player.y + 0x400, btype, player_id, Direction::Left, &state.constants);
                        state.create_caret(player.x - 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0xc00, player.y + 0x400, btype, player_id, Direction::Right, &state.constants);
                        state.create_caret(player.x + 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Right);
                    }
                    _ => {}
                }
            }

            state.sound_manager.play_sfx(34)
        }
    }

    fn tick_machine_gun(&mut self, player: &mut Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        const BULLETS: [u16; 3] = [10, 11, 12];

        if bullet_manager.count_bullets_multi(&BULLETS, player_id) >= 4 {
            return;
        }

        if player.controller.shoot() {
            self.counter2 = 0; // recharge time counter
            self.counter1 += 1; // autofire counter

            if self.counter1 > 5 {
                self.counter1 = 0;

                let btype = match self.level {
                    WeaponLevel::Level1 => 10,
                    WeaponLevel::Level2 => 11,
                    WeaponLevel::Level3 => 12,
                    WeaponLevel::None => {
                        unreachable!()
                    }
                };

                if !self.consume_ammo(1) {
                    state.sound_manager.play_sfx(37);
                    return;
                }

                if player.up {
                    if self.level == WeaponLevel::Level3 {
                        player.vel_y += 0x100;
                    }

                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0x600, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                            state.create_caret(player.x - 0x600, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0x600, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                            state.create_caret(player.x + 0x600, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                } else if player.down {
                    if self.level == WeaponLevel::Level3 {
                        if player.vel_y > 0 {
                            player.vel_y /= 2;
                        }
                        if player.vel_y > -0x400 {
                            player.vel_y = (player.vel_y - 0x200).max(-0x400);
                        }
                    }

                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0x600, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                            state.create_caret(player.x - 0x600, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0x600, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                            state.create_caret(player.x + 0x600, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                } else {
                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0x1800, player.y + 0x600, btype, player_id, Direction::Left, &state.constants);
                            state.create_caret(player.x - 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0x1800, player.y + 0x600, btype, player_id, Direction::Right, &state.constants);
                            state.create_caret(player.x + 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Right);
                        }
                        _ => {}
                    }
                }

                if self.level == WeaponLevel::Level3 {
                    state.sound_manager.play_sfx(49);
                } else {
                    state.sound_manager.play_sfx(32);
                }
            }
        } else {
            self.counter1 = 6;
            self.counter2 += 1;

            if (player.equip.has_turbocharge() && self.counter2 > 1) || self.counter2 > 4 {
                self.counter2 = 0;
                self.refill_ammo(1);
            }
        }
    }

    fn tick_blade(&mut self, player: &Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        const BULLETS: [u16; 3] = [25, 26, 27];

        if player.controller.trigger_shoot() && bullet_manager.count_bullets_multi(&BULLETS, player_id) == 0 {
            let btype = match self.level {
                WeaponLevel::Level1 => 25,
                WeaponLevel::Level2 => 26,
                WeaponLevel::Level3 => 27,
                WeaponLevel::None => {
                    unreachable!()
                }
            };

            if player.up {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y + 0x800, btype, player_id, Direction::Up, &state.constants);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y + 0x800, btype, player_id, Direction::Up, &state.constants);
                    }
                    _ => {}
                }
            } else if player.down {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y - 0xc00, btype, player_id, Direction::Bottom, &state.constants);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y - 0xc00, btype, player_id, Direction::Bottom, &state.constants);
                    }
                    _ => {}
                }
            } else {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x + 0xc00, player.y - 0x600, btype, player_id, Direction::Left, &state.constants);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x - 0xc00, player.y - 0x600, btype, player_id, Direction::Right, &state.constants);
                    }
                    _ => {}
                }
            }

            state.sound_manager.play_sfx(34)
        }
    }

    fn tick_nemesis(&mut self, player: &Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        const BULLETS: [u16; 3] = [34, 35, 36];

        if player.controller.trigger_shoot() && bullet_manager.count_bullets_multi(&BULLETS, player_id) < 2 {
            let btype = match self.level {
                WeaponLevel::Level1 => 34,
                WeaponLevel::Level2 => 35,
                WeaponLevel::Level3 => 36,
                WeaponLevel::None => {
                    unreachable!()
                }
            };

            if !self.consume_ammo(1) {
                state.sound_manager.play_sfx(37);
                return;
            }

            if player.up {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y - 0x1800, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x - 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y - 0x1800, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x + 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else if player.down {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y + 0x1800, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x - 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y + 0x1800, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x + 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x2c00, player.y + 0x600, btype, player_id, Direction::Left, &state.constants);
                        state.create_caret(player.x - 0x2000, player.y + 0x600, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x2c00, player.y + 0x600, btype, player_id, Direction::Right, &state.constants);
                        state.create_caret(player.x + 0x2000, player.y + 0x600, CaretType::Shoot, Direction::Right);
                    }
                    _ => {}
                }
            }

            match self.level {
                WeaponLevel::Level1 => state.sound_manager.play_sfx(117),
                WeaponLevel::Level2 => state.sound_manager.play_sfx(49),
                WeaponLevel::Level3 => state.sound_manager.play_sfx(60),
                _ => unreachable!(),
            }
        }
    }

    fn tick_spur(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        inventory: &mut Inventory,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        let mut shoot = false;
        let btype;

        if player.controller.shoot() {
            inventory.add_xp(if player.equip.has_turbocharge() { 3 } else { 2 }, player, state);
            self.counter1 += 1;

            if (self.counter1 / 2 % 2) != 0 {
                match self.level {
                    WeaponLevel::Level1 => {
                        state.sound_manager.play_sfx(59);
                    }
                    WeaponLevel::Level2 => {
                        state.sound_manager.play_sfx(60);
                    }
                    WeaponLevel::Level3 => {
                        if let (_, _, false) = inventory.get_current_max_exp(&state.constants) {
                            state.sound_manager.play_sfx(61);
                        }
                    }
                    WeaponLevel::None => unreachable!(),
                }
            }
        } else {
            if self.counter1 > 0 {
                shoot = true;
                self.counter1 = 0;
            }
        }

        if let (_, _, true) = inventory.get_current_max_exp(&state.constants) {
            if self.counter2 == 0 {
                self.counter2 = 1;
                state.sound_manager.play_sfx(65);
            }
        } else {
            self.counter2 = 0;
        }

        let level = self.level;
        if !player.controller.shoot() {
            inventory.reset_current_weapon_xp();
        }

        match level {
            WeaponLevel::Level1 => {
                btype = 6;
                shoot = false;
            }
            WeaponLevel::Level2 => btype = 37,
            WeaponLevel::Level3 => {
                if self.counter2 == 1 {
                    btype = 39;
                } else {
                    btype = 38;
                }
            }
            WeaponLevel::None => unreachable!(),
        }

        const BULLETS: [u16; 6] = [44, 45, 46, 47, 48, 49];
        if bullet_manager.count_bullets_multi(&BULLETS, player_id) == 0 && (player.controller.trigger_shoot() || shoot) {
            if !self.consume_ammo(1) {
                state.sound_manager.play_sfx(37);
            } else {
                if player.up {
                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0x200, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                            state.create_caret(player.x - 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0x200, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                            state.create_caret(player.x + 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                } else if player.down {
                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0x200, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                            state.create_caret(player.x - 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0x200, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                            state.create_caret(player.x + 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                } else {
                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0xc00, player.y + 0x600, btype, player_id, Direction::Left, &state.constants);
                            state.create_caret(player.x - 0xc00, player.y + 0x600, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0xc00, player.y + 0x600, btype, player_id, Direction::Right, &state.constants);
                            state.create_caret(player.x + 0xc00, player.y + 0x600, CaretType::Shoot, Direction::Right);
                        }
                        _ => {}
                    }
                }

                let sound = match btype {
                    6 => 49,
                    37 => 62,
                    38 => 63,
                    39 => 64,
                    _ => 0,
                };

                state.sound_manager.play_sfx(sound);
            }
        }
    }

    pub fn tick(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        inventory: &mut Inventory,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        if !player.cond.alive() || player.cond.hidden() {
            return;
        }

        // todo lua hook

        match self.wtype {
            WeaponType::None => {}
            WeaponType::Snake => self.tick_snake(player, player_id, bullet_manager, state),
            WeaponType::PolarStar => self.tick_polar_star(player, player_id, bullet_manager, state),
            WeaponType::Fireball => self.tick_fireball(player, player_id, bullet_manager, state),
            WeaponType::MachineGun => self.tick_machine_gun(player, player_id, bullet_manager, state),
            WeaponType::MissileLauncher => {}
            WeaponType::Bubbler => {}
            WeaponType::Blade => self.tick_blade(player, player_id, bullet_manager, state),
            WeaponType::SuperMissileLauncher => {}
            WeaponType::Nemesis => self.tick_nemesis(player, player_id, bullet_manager, state),
            WeaponType::Spur => self.tick_spur(player, player_id, inventory, bullet_manager, state),
        }
    }
}
