use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::BulletManager;
use crate::weapon::Weapon;

impl Weapon {
    pub(in crate::weapon) fn tick_missile_launcher(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {}
}
