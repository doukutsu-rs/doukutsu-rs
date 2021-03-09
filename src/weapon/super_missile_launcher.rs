use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::BulletManager;
use crate::weapon::{Weapon, WeaponLevel};

impl Weapon {
    pub(in crate::weapon) fn tick_super_missile_launcher(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 6] = [28, 29, 30, 31, 32, 33];
        
        let btype = match self.level {
            WeaponLevel::Level1 => 28,
            WeaponLevel::Level2 => 29,
            WeaponLevel::Level3 => 30,
            WeaponLevel::None => unreachable!(),
        };
    }
}
