use num_derive::FromPrimitive;

use crate::bullet::BulletManager;
use crate::caret::CaretType;
use crate::common::Direction;
use crate::player::Player;
use crate::SharedGameState;

#[derive(PartialEq, Eq, Copy, Clone, FromPrimitive)]
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
            WeaponLevel::None => { WeaponLevel::Level1 }
            WeaponLevel::Level1 => { WeaponLevel::Level2 }
            WeaponLevel::Level2 => { WeaponLevel::Level3 }
            WeaponLevel::Level3 => { WeaponLevel::Level3 }
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
}

impl Weapon {
    pub fn new(wtype: WeaponType, level: WeaponLevel, experience: u16, ammo: u16, max_ammo: u16) -> Weapon {
        Weapon {
            wtype,
            level,
            experience,
            ammo,
            max_ammo,
        }
    }

    pub fn consume_ammo(&mut self, ammo: u16) -> bool {
        if self.max_ammo == 0 {
            return true;
        }

        if self.ammo >= ammo {
            self.ammo -= ammo;
            return true;
        }

        false
    }

    pub fn shoot_bullet_polar_star(&mut self, player: &Player, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        if state.key_trigger.fire() && bullet_manager.count_bullets_multi([4, 5, 6]) < 2 {
            let btype = match self.level {
                WeaponLevel::Level1 => { 4 }
                WeaponLevel::Level2 => { 5 }
                WeaponLevel::Level3 => { 6 }
                WeaponLevel::None => { unreachable!() }
            };

            if !self.consume_ammo(1) {
                // todo: play sound 37
                return;
            }

            if player.up {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y - 8 * 0x200, btype, Direction::Up, &state.constants);
                        state.create_caret(player.x - 0x200, player.y - 8 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y - 8 * 0x200, btype, Direction::Up, &state.constants);
                        state.create_caret(player.x + 0x200, player.y - 8 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    _ => { unreachable!() }
                }
            } else if player.down {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x200, player.y + 8 * 0x200, btype, Direction::Bottom, &state.constants);
                        state.create_caret(player.x - 0x200, player.y + 8 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x200, player.y + 8 * 0x200, btype, Direction::Bottom, &state.constants);
                        state.create_caret(player.x + 0x200, player.y + 8 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    _ => { unreachable!() }
                }
            } else {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 6 * 0x200, player.y + 3 * 0x200, btype, Direction::Left, &state.constants);
                        state.create_caret(player.x - 6 * 0x200, player.y + 3 * 0x200, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 6 * 0x200, player.y + 3 * 0x200, btype, Direction::Right, &state.constants);
                        state.create_caret(player.x + 6 * 0x200, player.y + 3 * 0x200, CaretType::Shoot, Direction::Right);
                    }
                    _ => { unreachable!() }
                }
            }

            if self.level == WeaponLevel::Level3 {
                // todo play sound 49
            } else {
                // todo play sound 32
            }
        }
    }

    pub fn shoot_bullet(&mut self, player: &Player, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        if player.cond.hidden() {
            return;
        }

        match self.wtype {
            WeaponType::None => {}
            WeaponType::Snake => {}
            WeaponType::PolarStar => { self.shoot_bullet_polar_star(player, bullet_manager, state) }
            WeaponType::Fireball => {}
            WeaponType::MachineGun => {}
            WeaponType::MissileLauncher => {}
            WeaponType::Bubbler => {}
            WeaponType::Blade => {}
            WeaponType::SuperMissileLauncher => {}
            WeaponType::Nemesis => {}
            WeaponType::Spur => {}
        }
    }
}
