use num_traits::abs;

use crate::npc::NPC;
use crate::player::Player;

impl NPC {
    pub fn get_closest_player_idx_mut<'a>(&self, players: &[&'a mut Player; 2]) -> usize {
        let mut max_dist = f64::MAX;
        let mut player_idx = 0;

        for (idx, player) in players.iter().enumerate() {
            if !player.cond.alive() || player.cond.hidden() {
                continue;
            }

            let dist_x = abs(self.x - player.x) as f64;
            let dist_y = abs(self.y - player.y) as f64;
            let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

            if dist < max_dist {
                max_dist = dist;
                player_idx = idx;
            }
        }

        player_idx
    }

    pub fn get_closest_player_mut<'a>(&self, players: [&'a mut Player; 2]) -> &'a mut Player {
        let mut max_dist = f64::MAX;
        let mut player_idx = 0;

        for (idx, player) in players.iter().enumerate() {
            if !player.cond.alive() || player.cond.hidden() {
                continue;
            }

            let dist_x = abs(self.x - player.x) as f64;
            let dist_y = abs(self.y - player.y) as f64;
            let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

            if dist < max_dist {
                max_dist = dist;
                player_idx = idx;
            }
        }

        players[player_idx]
    }
}
