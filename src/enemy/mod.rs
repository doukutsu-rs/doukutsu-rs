use crate::bitfield;

pub mod critter;

bitfield! {
  pub struct NPCCond(u16);
  impl Debug;

  pub damage_boss, set_damage_boss: 4;
  pub alive, set_alive: 7;
}

bitfield! {
  pub struct NPCFlags(u16);
  impl Debug;

  pub solid_soft, set_solid_soft: 0;
  pub ignore_tile_44, set_ignore_tile_44: 1;
  pub invulnerable, set_invulnerable: 2;
  pub ignore_solidity, set_ignore_solidity: 3;
  pub bouncy, set_bouncy: 4;
  pub shootable, set_shootable: 5;
  pub solid_hard, set_solid_hard: 6;
  pub rear_and_top_not_hurt, set_rear_and_top_not_hurt: 7;
  pub event_when_touched, set_event_when_touched: 8;
  pub event_when_killed, set_event_when_killed: 9;
  pub flag_x400, set_flag_x400: 10;
  pub appear_when_flag_set, set_appear_when_flag_set: 11;
  pub spawn_in_other_direction, set_spawn_in_other_direction: 12;
  pub interactable, set_interactable: 13;
  pub hide_when_flag_set, set_hide_when_flag_set: 14;
  pub show_damage, set_show_damage: 15;
}
