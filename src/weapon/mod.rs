use num_derive::FromPrimitive;

use crate::caret::CaretType;
use crate::common::Direction;
use crate::inventory::Inventory;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::{Bullet, BulletManager};

mod blade;
mod bubbler;
pub mod bullet;
mod fireball;
mod machine_gun;
mod missile_launcher;
mod nemesis;
mod polar_star;
mod snake;
mod spur;
mod super_missile_launcher;

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
            WeaponType::MissileLauncher => self.tick_missile_launcher(player, player_id, bullet_manager, state),
            WeaponType::Bubbler => self.tick_bubbler(player, player_id, bullet_manager, state),
            WeaponType::Blade => self.tick_blade(player, player_id, bullet_manager, state),
            WeaponType::SuperMissileLauncher => self.tick_super_missile_launcher(player, player_id, bullet_manager, state),
            WeaponType::Nemesis => self.tick_nemesis(player, player_id, bullet_manager, state),
            WeaponType::Spur => self.tick_spur(player, player_id, inventory, bullet_manager, state),
        }
    }
}
