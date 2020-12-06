use num_traits::abs;

use crate::npc::NPC;
use crate::player::Player;

impl NPC {
    pub fn animate(&mut self, ticks_between_frames: u16, start_frame: u16, end_frame: u16) {
        self.anim_counter += 1;
        if self.anim_counter > ticks_between_frames {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > end_frame {
                self.anim_num = start_frame;
            }
        }
    }

    /// Returns index of player that's closest to the current NPC.
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

    /// Returns a reference to closest player.
    pub fn get_closest_player_mut<'a>(&self, players: [&'a mut Player; 2]) -> &'a mut Player {
        let idx = self.get_closest_player_idx_mut(&players);

        players[idx]
    }
}
