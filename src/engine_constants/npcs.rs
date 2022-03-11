use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Index;

use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::common::Rect;
use crate::macros::fmt::Formatter;

#[derive(Copy, Clone)]
pub struct SafeNPCRect<const T: usize>(pub [Rect<u16>; T]);

impl<const T: usize> Serialize for SafeNPCRect<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

struct SafeNPCRectArrayVisitor<const T: usize>(pub PhantomData<[Rect<u16>; T]>);

impl<'de, const T: usize> Visitor<'de> for SafeNPCRectArrayVisitor<T> {
    type Value = [Rect<u16>; T];

    fn expecting(&self, formatter: &mut Formatter) -> crate::macros::fmt::Result {
        formatter.write_str("an array of rectangles")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut rects = [Rect::default(); T];
        for (i, rect) in rects.iter_mut().enumerate() {
            let el: Rect<u16> = seq.next_element()?.ok_or_else(|| Error::invalid_length(i, &"rectangle"))?;
            *rect = el;
        }

        Ok(rects)
    }
}

impl<'de, const T: usize> Deserialize<'de> for SafeNPCRect<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SafeNPCRectArrayVisitor(PhantomData)).map(SafeNPCRect)
    }
}

static EMPTY_RECT: Rect<u16> = Rect { left: 0, top: 0, right: 0, bottom: 0 };

impl<const T: usize> Index<usize> for SafeNPCRect<T> {
    type Output = Rect<u16>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.0.get(index).unwrap_or(&EMPTY_RECT)
    }
}

impl<const T: usize> Debug for SafeNPCRect<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> crate::macros::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct NPCConsts {
    // pub n000_null: () // Defined in code
    #[serde(default = "default_n001_experience")]
    pub n001_experience: SafeNPCRect<6>,

    #[serde(default = "default_n002_behemoth")]
    pub n002_behemoth: SafeNPCRect<14>,

    // pub n003_dead_enemy: () // Defined in code
    #[serde(default = "default_n004_smoke")]
    pub n004_smoke: SafeNPCRect<16>,

    #[serde(default = "default_n005_green_critter")]
    pub n005_green_critter: SafeNPCRect<6>,

    #[serde(default = "default_n006_green_beetle")]
    pub n006_green_beetle: SafeNPCRect<10>,

    #[serde(default = "default_n007_basil")]
    pub n007_basil: SafeNPCRect<6>,

    #[serde(default = "default_n008_blue_beetle")]
    pub n008_blue_beetle: SafeNPCRect<4>,

    #[serde(default = "default_n009_balrog_falling_in")]
    pub n009_balrog_falling_in: SafeNPCRect<6>,

    #[serde(default = "default_n010_balrog_shooting")]
    pub n010_balrog_shooting: SafeNPCRect<8>,

    #[serde(default = "default_n011_balrog_energy_shot")]
    pub n011_balrog_energy_shot: SafeNPCRect<3>,

    #[serde(default = "default_n012_balrog_cutscene")]
    pub n012_balrog_cutscene: SafeNPCRect<28>,

    #[serde(default = "default_n013_forcefield")]
    pub n013_forcefield: SafeNPCRect<4>,

    #[serde(default = "default_n014_key")]
    pub n014_key: SafeNPCRect<3>,

    #[serde(default = "default_n015_closed_chest")]
    pub n015_closed_chest: SafeNPCRect<3>,

    #[serde(default = "default_n016_save_point")]
    pub n016_save_point: SafeNPCRect<8>,

    #[serde(default = "default_n017_health_refill")]
    pub n017_health_refill: SafeNPCRect<2>,

    #[serde(default = "default_n018_door")]
    pub n018_door: SafeNPCRect<2>,

    #[serde(default = "default_n019_balrog_bust_in")]
    pub n019_balrog_bust_in: SafeNPCRect<8>,

    #[serde(default = "default_n020_computer")]
    pub n020_computer: SafeNPCRect<4>,

    #[serde(default = "default_n021_chest_open")]
    pub n021_chest_open: Rect<u16>,

    #[serde(default = "default_n022_teleporter")]
    pub n022_teleporter: SafeNPCRect<2>,

    #[serde(default = "default_n023_teleporter_lights")]
    pub n023_teleporter_lights: SafeNPCRect<8>,

    #[serde(default = "default_n024_power_critter")]
    pub n024_power_critter: SafeNPCRect<12>,

    #[serde(default = "default_n025_lift")]
    pub n025_lift: SafeNPCRect<2>,

    #[serde(default = "default_n026_bat_flying")]
    pub n026_bat_flying: SafeNPCRect<8>,

    #[serde(default = "default_n027_death_trap")]
    pub n027_death_trap: Rect<u16>,

    #[serde(default = "default_n028_flying_critter")]
    pub n028_flying_critter: SafeNPCRect<12>,

    #[serde(default = "default_n029_cthulhu")]
    pub n029_cthulhu: SafeNPCRect<4>,

    #[serde(default = "default_n030_hermit_gunsmith")]
    pub n030_hermit_gunsmith: SafeNPCRect<3>,

    #[serde(default = "default_n031_bat_hanging")]
    pub n031_bat_hanging: SafeNPCRect<10>,

    #[serde(default = "default_n032_life_capsule")]
    pub n032_life_capsule: SafeNPCRect<2>,

    #[serde(default = "default_n033_balrog_bouncing_projectile")]
    pub n033_balrog_bouncing_projectile: SafeNPCRect<2>,

    #[serde(default = "default_n034_bed")]
    pub n034_bed: SafeNPCRect<2>,

    #[serde(default = "default_n035_mannan")]
    pub n035_mannan: SafeNPCRect<8>,

    #[serde(default = "default_n036_balrog_hover")]
    pub n036_balrog_hover: SafeNPCRect<12>,

    #[serde(default = "default_n037_sign")]
    pub n037_sign: SafeNPCRect<2>,

    #[serde(default = "default_n038_fireplace")]
    pub n038_fireplace: SafeNPCRect<4>,

    #[serde(default = "default_n039_save_sign")]
    pub n039_save_sign: SafeNPCRect<2>,

    #[serde(default = "default_n040_santa")]
    pub n040_santa: SafeNPCRect<14>,

    #[serde(default = "default_n041_busted_door")]
    pub n041_busted_door: Rect<u16>,

    #[serde(default = "default_n042_sue")]
    pub n042_sue: SafeNPCRect<26>,

    #[serde(default = "default_n043_chalkboard")]
    pub n043_chalkboard: SafeNPCRect<2>,

    #[serde(default = "default_n044_polish")]
    pub n044_polish: SafeNPCRect<6>,

    #[serde(default = "default_n045_baby")]
    pub n045_baby: SafeNPCRect<3>,

    // pub n046_hv_trigger: () // Defined in code
    #[serde(default = "default_n047_sandcroc")]
    pub n047_sandcroc: SafeNPCRect<5>,

    #[serde(default = "default_n048_omega_projectiles")]
    pub n048_omega_projectiles: SafeNPCRect<4>,

    #[serde(default = "default_n049_skullhead")]
    pub n049_skullhead: SafeNPCRect<6>,

    #[serde(default = "default_n050_skeleton_projectile")]
    pub n050_skeleton_projectile: SafeNPCRect<4>,

    #[serde(default = "default_n051_crow_and_skullhead")]
    pub n051_crow_and_skullhead: SafeNPCRect<10>,

    #[serde(default = "default_n052_sitting_blue_robot")]
    pub n052_sitting_blue_robot: Rect<u16>,

    #[serde(default = "default_n053_skullstep_leg")]
    pub n053_skullstep_leg: SafeNPCRect<4>,

    #[serde(default = "default_n054_skullstep")]
    pub n054_skullstep: SafeNPCRect<6>,

    #[serde(default = "default_n055_kazuma")]
    pub n055_kazuma: SafeNPCRect<12>,

    #[serde(default = "default_n056_tan_beetle")]
    pub n056_tan_beetle: SafeNPCRect<6>,

    #[serde(default = "default_n057_crow")]
    pub n057_crow: SafeNPCRect<10>,

    #[serde(default = "default_n058_basu")]
    pub n058_basu: SafeNPCRect<6>,

    #[serde(default = "default_n059_eye_door")]
    pub n059_eye_door: SafeNPCRect<4>,

    #[serde(default = "default_n060_toroko")]
    pub n060_toroko: SafeNPCRect<16>,

    #[serde(default = "default_n061_king")]
    pub n061_king: SafeNPCRect<22>,

    #[serde(default = "default_n062_kazuma_computer")]
    pub n062_kazuma_computer: SafeNPCRect<3>,

    #[serde(default = "default_n063_toroko_stick")]
    pub n063_toroko_stick: SafeNPCRect<12>,

    #[serde(default = "default_n064_first_cave_critter")]
    pub n064_first_cave_critter: SafeNPCRect<6>,

    #[serde(default = "default_n065_first_cave_bat")]
    pub n065_first_cave_bat: SafeNPCRect<8>,

    #[serde(default = "default_n066_misery_bubble")]
    pub n066_misery_bubble: SafeNPCRect<4>,

    #[serde(default = "default_n067_misery_floating")]
    pub n067_misery_floating: SafeNPCRect<16>,

    #[serde(default = "default_n068_balrog_running")]
    pub n068_balrog_running: SafeNPCRect<18>,

    #[serde(default = "default_n069_pignon")]
    pub n069_pignon: SafeNPCRect<12>,

    #[serde(default = "default_n070_sparkle")]
    pub n070_sparkle: SafeNPCRect<4>,

    #[serde(default = "default_n071_chinfish")]
    pub n071_chinfish: SafeNPCRect<6>,

    #[serde(default = "default_n072_sprinkler")]
    pub n072_sprinkler: SafeNPCRect<2>,

    #[serde(default = "default_n073_water_droplet")]
    pub n073_water_droplet: SafeNPCRect<5>,

    #[serde(default = "default_n074_jack")]
    pub n074_jack: SafeNPCRect<12>,

    #[serde(default = "default_n075_kanpachi")]
    pub n075_kanpachi: SafeNPCRect<2>,

    // pub n076_flowers: () // Defined in code
    #[serde(default = "default_n077_yamashita")]
    pub n077_yamashita: SafeNPCRect<3>,

    #[serde(default = "default_n078_pot")]
    pub n078_pot: SafeNPCRect<2>,

    #[serde(default = "default_n079_mahin")]
    pub n079_mahin: SafeNPCRect<6>,

    #[serde(default = "default_n080_gravekeeper")]
    pub n080_gravekeeper: SafeNPCRect<14>,

    #[serde(default = "default_n081_giant_pignon")]
    pub n081_giant_pignon: SafeNPCRect<12>,

    #[serde(default = "default_n082_misery_standing")]
    pub n082_misery_standing: SafeNPCRect<18>,

    #[serde(default = "default_n083_igor_cutscene")]
    pub n083_igor_cutscene: SafeNPCRect<16>,

    #[serde(default = "default_n084_basu_projectile")]
    pub n084_basu_projectile: SafeNPCRect<4>,

    #[serde(default = "default_n085_terminal")]
    pub n085_terminal: SafeNPCRect<6>,

    #[serde(default = "default_n086_missile_pickup")]
    pub n086_missile_pickup: SafeNPCRect<5>,

    #[serde(default = "default_n087_heart_pickup")]
    pub n087_heart_pickup: SafeNPCRect<5>,

    #[serde(default = "default_n088_igor_boss")]
    pub n088_igor_boss: SafeNPCRect<24>,

    #[serde(default = "default_n089_igor_dead")]
    pub n089_igor_dead: SafeNPCRect<8>,

    #[serde(default = "default_n090_background")]
    pub n090_background: Rect<u16>,

    #[serde(default = "default_n091_mimiga_cage")]
    pub n091_mimiga_cage: Rect<u16>,

    #[serde(default = "default_n092_sue_at_pc")]
    pub n092_sue_at_pc: SafeNPCRect<3>,

    #[serde(default = "default_n093_chaco")]
    pub n093_chaco: SafeNPCRect<14>,

    #[serde(default = "default_n094_kulala")]
    pub n094_kulala: SafeNPCRect<5>,

    #[serde(default = "default_n095_jelly")]
    pub n095_jelly: SafeNPCRect<8>,

    #[serde(default = "default_n096_fan_left")]
    pub n096_fan_left: SafeNPCRect<3>,

    #[serde(default = "default_n097_fan_up")]
    pub n097_fan_up: SafeNPCRect<3>,

    #[serde(default = "default_n098_fan_right")]
    pub n098_fan_right: SafeNPCRect<3>,

    #[serde(default = "default_n099_fan_down")]
    pub n099_fan_down: SafeNPCRect<3>,

    #[serde(default = "default_n100_grate")]
    pub n100_grate: SafeNPCRect<2>,

    #[serde(default = "default_n101_malco_screen")]
    pub n101_malco_screen: SafeNPCRect<3>,

    #[serde(default = "default_n102_malco_computer_wave")]
    pub n102_malco_computer_wave: SafeNPCRect<4>,

    #[serde(default = "default_n103_mannan_projectile")]
    pub n103_mannan_projectile: SafeNPCRect<6>,

    #[serde(default = "default_n104_frog")]
    pub n104_frog: SafeNPCRect<6>,

    #[serde(default = "default_n105_hey_bubble_low")]
    pub n105_hey_bubble_low: SafeNPCRect<2>,

    // pub n106_hey_bubble_high: () // Defined in code
    #[serde(default = "default_n107_malco_broken")]
    pub n107_malco_broken: SafeNPCRect<10>,

    #[serde(default = "default_n108_balfrog_projectile")]
    pub n108_balfrog_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n109_malco_powered_on")]
    pub n109_malco_powered_on: SafeNPCRect<4>,

    #[serde(default = "default_n110_puchi")]
    pub n110_puchi: SafeNPCRect<6>,

    #[serde(default = "default_n111_quote_teleport_out")]
    pub n111_quote_teleport_out: SafeNPCRect<4>,

    #[serde(default = "default_n112_quote_teleport_in")]
    pub n112_quote_teleport_in: SafeNPCRect<4>,

    #[serde(default = "default_n113_professor_booster")]
    pub n113_professor_booster: SafeNPCRect<14>,

    #[serde(default = "default_n114_press")]
    pub n114_press: SafeNPCRect<3>,

    #[serde(default = "default_n115_ravil")]
    pub n115_ravil: SafeNPCRect<12>,

    #[serde(default = "default_n116_red_petals")]
    pub n116_red_petals: Rect<u16>,

    #[serde(default = "default_n117_curly")]
    pub n117_curly: SafeNPCRect<20>,

    #[serde(default = "default_n118_curly_boss")]
    pub n118_curly_boss: SafeNPCRect<18>,

    #[serde(default = "default_n119_table_chair")]
    pub n119_table_chair: Rect<u16>,

    #[serde(default = "default_n120_colon_a")]
    pub n120_colon_a: SafeNPCRect<2>,

    #[serde(default = "default_n121_colon_b")]
    pub n121_colon_b: SafeNPCRect<3>,

    #[serde(default = "default_n122_colon_enraged")]
    pub n122_colon_enraged: SafeNPCRect<20>,

    #[serde(default = "default_n123_curly_boss_bullet")]
    pub n123_curly_boss_bullet: SafeNPCRect<4>,

    #[serde(default = "default_n124_sunstone")]
    pub n124_sunstone: SafeNPCRect<2>,

    #[serde(default = "default_n125_hidden_item")]
    pub n125_hidden_item: SafeNPCRect<2>,

    #[serde(default = "default_n126_puppy_running")]
    pub n126_puppy_running: SafeNPCRect<12>,

    #[serde(default = "default_n127_machine_gun_trail_l2")]
    pub n127_machine_gun_trail_l2: SafeNPCRect<6>,

    #[serde(default = "default_n128_machine_gun_trail_l3")]
    pub n128_machine_gun_trail_l3: SafeNPCRect<20>,

    #[serde(default = "default_n129_fireball_snake_trail")]
    pub n129_fireball_snake_trail: SafeNPCRect<18>,

    #[serde(default = "default_n130_puppy_sitting")]
    pub n130_puppy_sitting: SafeNPCRect<8>,

    #[serde(default = "default_n131_puppy_sleeping")]
    pub n131_puppy_sleeping: SafeNPCRect<2>,

    #[serde(default = "default_n132_puppy_barking")]
    pub n132_puppy_barking: SafeNPCRect<10>,

    #[serde(default = "default_n133_jenka")]
    pub n133_jenka: SafeNPCRect<4>,

    #[serde(default = "default_n134_armadillo")]
    pub n134_armadillo: SafeNPCRect<6>,

    #[serde(default = "default_n135_skeleton")]
    pub n135_skeleton: SafeNPCRect<4>,

    #[serde(default = "default_n136_puppy_carried")]
    pub n136_puppy_carried: SafeNPCRect<4>,

    #[serde(default = "default_n137_large_door_frame")]
    pub n137_large_door_frame: Rect<u16>,

    #[serde(default = "default_n138_large_door")]
    pub n138_large_door: SafeNPCRect<2>,

    #[serde(default = "default_n139_doctor")]
    pub n139_doctor: SafeNPCRect<6>,

    #[serde(default = "default_n140_toroko_frenzied")]
    pub n140_toroko_frenzied: SafeNPCRect<28>,

    #[serde(default = "default_n141_toroko_block_projectile")]
    pub n141_toroko_block_projectile: SafeNPCRect<2>,

    #[serde(default = "default_n142_flower_cub")]
    pub n142_flower_cub: SafeNPCRect<5>,

    #[serde(default = "default_n143_jenka_collapsed")]
    pub n143_jenka_collapsed: SafeNPCRect<2>,

    #[serde(default = "default_n144_toroko_teleporting_in")]
    pub n144_toroko_teleporting_in: SafeNPCRect<10>,

    #[serde(default = "default_n145_king_sword")]
    pub n145_king_sword: SafeNPCRect<2>,

    #[serde(default = "default_n146_lightning")]
    pub n146_lightning: SafeNPCRect<5>,

    #[serde(default = "default_n147_critter_purple")]
    pub n147_critter_purple: SafeNPCRect<12>,

    #[serde(default = "default_n148_critter_purple_projectile")]
    pub n148_critter_purple_projectile: SafeNPCRect<2>,

    #[serde(default = "default_n149_horizontal_moving_block")]
    pub n149_horizontal_moving_block: Rect<u16>,

    #[serde(default = "default_n150_quote")]
    pub n150_quote: SafeNPCRect<20>,

    #[serde(default = "default_n151_blue_robot_standing")]
    pub n151_blue_robot_standing: SafeNPCRect<4>,

    // pub n152_shutter_stuck: () // Defined in code
    #[serde(default = "default_n153_gaudi")]
    pub n153_gaudi: SafeNPCRect<14>,

    #[serde(default = "default_n154_gaudi_dead")]
    pub n154_gaudi_dead: SafeNPCRect<6>,

    #[serde(default = "default_n155_gaudi_flying")]
    pub n155_gaudi_flying: SafeNPCRect<8>,

    #[serde(default = "default_n156_gaudi_projectile")]
    pub n156_gaudi_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n157_vertical_moving_block")]
    pub n157_vertical_moving_block: Rect<u16>,

    #[serde(default = "default_n158_fish_missile")]
    pub n158_fish_missile: SafeNPCRect<8>,

    #[serde(default = "default_n159_monster_x_defeated")]
    pub n159_monster_x_defeated: Rect<u16>,

    #[serde(default = "default_n160_puu_black")]
    pub n160_puu_black: SafeNPCRect<8>,

    #[serde(default = "default_n161_puu_black_projectile")]
    pub n161_puu_black_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n162_puu_black_dead")]
    pub n162_puu_black_dead: SafeNPCRect<3>,

    #[serde(default = "default_n163_dr_gero")]
    pub n163_dr_gero: SafeNPCRect<4>,

    #[serde(default = "default_n164_nurse_hasumi")]
    pub n164_nurse_hasumi: SafeNPCRect<4>,

    #[serde(default = "default_n165_curly_collapsed")]
    pub n165_curly_collapsed: SafeNPCRect<3>,

    #[serde(default = "default_n166_chaba")]
    pub n166_chaba: SafeNPCRect<2>,

    #[serde(default = "default_n167_booster_falling")]
    pub n167_booster_falling: SafeNPCRect<3>,

    #[serde(default = "default_n168_boulder")]
    pub n168_boulder: Rect<u16>,

    #[serde(default = "default_n169_balrog_shooting_missiles")]
    pub n169_balrog_shooting_missiles: SafeNPCRect<18>,

    #[serde(default = "default_n170_balrog_missile")]
    pub n170_balrog_missile: SafeNPCRect<4>,

    #[serde(default = "default_n171_fire_whirrr")]
    pub n171_fire_whirrr: SafeNPCRect<4>,

    #[serde(default = "default_n172_fire_whirrr_projectile")]
    pub n172_fire_whirrr_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n173_gaudi_armored")]
    pub n173_gaudi_armored: SafeNPCRect<8>,

    #[serde(default = "default_n174_gaudi_armored_projectile")]
    pub n174_gaudi_armored_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n175_gaudi_egg")]
    pub n175_gaudi_egg: SafeNPCRect<4>,

    #[serde(default = "default_n176_buyo_buyo_base")]
    pub n176_buyo_buyo_base: SafeNPCRect<6>,

    #[serde(default = "default_n177_buyo_buyo")]
    pub n177_buyo_buyo: SafeNPCRect<2>,

    #[serde(default = "default_n178_core_blade_projectile")]
    pub n178_core_blade_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n179_core_wisp_projectile")]
    pub n179_core_wisp_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n180_curly_ai")]
    pub n180_curly_ai: SafeNPCRect<22>,

    #[serde(default = "default_n181_curly_ai_machine_gun")]
    pub n181_curly_ai_machine_gun: SafeNPCRect<4>,

    #[serde(default = "default_n182_curly_ai_polar_star")]
    pub n182_curly_ai_polar_star: SafeNPCRect<4>,

    #[serde(default = "default_n183_curly_air_tank_bubble")]
    pub n183_curly_air_tank_bubble: SafeNPCRect<2>,

    #[serde(default = "default_n184_shutter")]
    pub n184_shutter: SafeNPCRect<4>,

    #[serde(default = "default_n185_small_shutter")]
    pub n185_small_shutter: Rect<u16>,

    #[serde(default = "default_n186_lift_block")]
    pub n186_lift_block: SafeNPCRect<4>,

    #[serde(default = "default_n187_fuzz_core")]
    pub n187_fuzz_core: SafeNPCRect<4>,

    #[serde(default = "default_n188_fuzz")]
    pub n188_fuzz: SafeNPCRect<4>,

    #[serde(default = "default_n189_homing_flame")]
    pub n189_homing_flame: SafeNPCRect<3>,

    #[serde(default = "default_n190_broken_robot")]
    pub n190_broken_robot: SafeNPCRect<2>,

    // pub n191_water_level: () // Defined in code
    #[serde(default = "default_n192_scooter")]
    pub n192_scooter: SafeNPCRect<4>,

    #[serde(default = "default_n193_broken_scooter")]
    pub n193_broken_scooter: Rect<u16>,

    #[serde(default = "default_n194_broken_blue_robot")]
    pub n194_broken_blue_robot: Rect<u16>,

    #[serde(default = "default_n195_background_grate")]
    pub n195_background_grate: Rect<u16>,

    #[serde(default = "default_n196_ironhead_wall")]
    pub n196_ironhead_wall: SafeNPCRect<2>,

    #[serde(default = "default_n197_porcupine_fish")]
    pub n197_porcupine_fish: SafeNPCRect<4>,

    #[serde(default = "default_n198_ironhead_projectile")]
    pub n198_ironhead_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n199_wind_particles")]
    pub n199_wind_particles: SafeNPCRect<5>,

    #[serde(default = "default_n200_zombie_dragon")]
    pub n200_zombie_dragon: SafeNPCRect<12>,

    #[serde(default = "default_n201_zombie_dragon_dead")]
    pub n201_zombie_dragon_dead: SafeNPCRect<2>,

    #[serde(default = "default_n202_zombie_dragon_projectile")]
    pub n202_zombie_dragon_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n203_critter_destroyed_egg_corridor")]
    pub n203_critter_destroyed_egg_corridor: SafeNPCRect<6>,

    #[serde(default = "default_n204_small_falling_spike")]
    pub n204_small_falling_spike: SafeNPCRect<2>,

    #[serde(default = "default_n205_large_falling_spike")]
    pub n205_large_falling_spike: SafeNPCRect<2>,

    #[serde(default = "default_n206_counter_bomb")]
    pub n206_counter_bomb: SafeNPCRect<3>,

    #[serde(default = "default_n207_counter_bomb_countdown")]
    pub n207_counter_bomb_countdown: SafeNPCRect<5>,

    #[serde(default = "default_n208_basu_destroyed_egg_corridor")]
    pub n208_basu_destroyed_egg_corridor: SafeNPCRect<6>,

    #[serde(default = "default_n209_basu_projectile_destroyed_egg_corridor")]
    pub n209_basu_projectile_destroyed_egg_corridor: SafeNPCRect<4>,

    #[serde(default = "default_n210_beetle_destroyed_egg_corridor")]
    pub n210_beetle_destroyed_egg_corridor: SafeNPCRect<4>,

    #[serde(default = "default_n211_small_spikes")]
    pub n211_small_spikes: SafeNPCRect<4>,

    #[serde(default = "default_n212_sky_dragon")]
    pub n212_sky_dragon: SafeNPCRect<4>,

    #[serde(default = "default_n213_night_spirit")]
    pub n213_night_spirit: SafeNPCRect<10>,

    #[serde(default = "default_n214_night_spirit_projectile")]
    pub n214_night_spirit_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n215_sandcroc_outer_wall")]
    pub n215_sandcroc_outer_wall: SafeNPCRect<5>,

    #[serde(default = "default_n216_debug_cat")]
    pub n216_debug_cat: Rect<u16>,

    #[serde(default = "default_n217_itoh")]
    pub n217_itoh: SafeNPCRect<8>,

    #[serde(default = "default_n218_core_giant_ball")]
    pub n218_core_giant_ball: SafeNPCRect<2>,

    // pub n219_smoke_generator: () // Defined in code
    #[serde(default = "default_n220_shovel_brigade")]
    pub n220_shovel_brigade: SafeNPCRect<4>,

    #[serde(default = "default_n221_shovel_brigade_walking")]
    pub n221_shovel_brigade_walking: SafeNPCRect<12>,

    #[serde(default = "default_n222_prison_bars")]
    pub n222_prison_bars: Rect<u16>,

    #[serde(default = "default_n223_momorin")]
    pub n223_momorin: SafeNPCRect<6>,

    #[serde(default = "default_n224_chie")]
    pub n224_chie: SafeNPCRect<4>,

    #[serde(default = "default_n225_megane")]
    pub n225_megane: SafeNPCRect<4>,

    #[serde(default = "default_n226_kanpachi_plantation")]
    pub n226_kanpachi_plantation: SafeNPCRect<7>,

    #[serde(default = "default_n227_bucket")]
    pub n227_bucket: Rect<u16>,

    #[serde(default = "default_n228_droll")]
    pub n228_droll: SafeNPCRect<8>,

    #[serde(default = "default_n229_red_flowers_sprouts")]
    pub n229_red_flowers_sprouts: SafeNPCRect<2>,

    #[serde(default = "default_n230_red_flowers_blooming")]
    pub n230_red_flowers_blooming: SafeNPCRect<2>,

    #[serde(default = "default_n231_rocket")]
    pub n231_rocket: SafeNPCRect<2>,

    #[serde(default = "default_n232_orangebell")]
    pub n232_orangebell: SafeNPCRect<6>,

    #[serde(default = "default_n233_orangebell_bat")]
    pub n233_orangebell_bat: SafeNPCRect<8>,

    #[serde(default = "default_n234_red_flowers_picked")]
    pub n234_red_flowers_picked: SafeNPCRect<2>,

    #[serde(default = "default_n235_midorin")]
    pub n235_midorin: SafeNPCRect<8>,

    #[serde(default = "default_n236_gunfish")]
    pub n236_gunfish: SafeNPCRect<12>,

    #[serde(default = "default_n237_gunfish_projectile")]
    pub n237_gunfish_projectile: Rect<u16>,

    #[serde(default = "default_n238_press_sideways")]
    pub n238_press_sideways: SafeNPCRect<3>,

    #[serde(default = "default_n239_cage_bars")]
    pub n239_cage_bars: SafeNPCRect<2>,

    #[serde(default = "default_n240_mimiga_jailed")]
    pub n240_mimiga_jailed: SafeNPCRect<12>,

    #[serde(default = "default_n241_critter_red")]
    pub n241_critter_red: SafeNPCRect<6>,

    #[serde(default = "default_n242_bat_last_cave")]
    pub n242_bat_last_cave: SafeNPCRect<8>,

    // pub n243_bat_generator: () // Defined in code
    #[serde(default = "default_n244_lava_drop")]
    pub n244_lava_drop: Rect<u16>,

    #[serde(default = "default_n245_lava_drop_generator")]
    pub n245_lava_drop_generator: SafeNPCRect<4>,

    #[serde(default = "default_n246_press_proximity")]
    pub n246_press_proximity: SafeNPCRect<3>,

    #[serde(default = "default_n247_misery_boss")]
    pub n247_misery_boss: SafeNPCRect<18>,

    #[serde(default = "default_n248_misery_boss_vanishing")]
    pub n248_misery_boss_vanishing: SafeNPCRect<3>,

    #[serde(default = "default_n249_misery_boss_appearing")]
    pub n249_misery_boss_appearing: SafeNPCRect<2>,

    #[serde(default = "default_n250_misery_boss_lightning_ball")]
    pub n250_misery_boss_lightning_ball: SafeNPCRect<3>,

    #[serde(default = "default_n251_misery_boss_lightning")]
    pub n251_misery_boss_lightning: SafeNPCRect<2>,

    #[serde(default = "default_n252_misery_boss_bats")]
    pub n252_misery_boss_bats: SafeNPCRect<8>,

    #[serde(default = "default_n253_experience_capsule")]
    pub n253_experience_capsule: SafeNPCRect<2>,

    #[serde(default = "default_n254_helicopter")]
    pub n254_helicopter: SafeNPCRect<2>,

    #[serde(default = "default_n255_helicopter_blades")]
    pub n255_helicopter_blades: SafeNPCRect<8>,

    #[serde(default = "default_n256_doctor_facing_away")]
    pub n256_doctor_facing_away: SafeNPCRect<6>,

    #[serde(default = "default_n257_red_crystal")]
    pub n257_red_crystal: SafeNPCRect<3>,

    #[serde(default = "default_n258_mimiga_sleeping")]
    pub n258_mimiga_sleeping: Rect<u16>,

    #[serde(default = "default_n259_curly_unconscious")]
    pub n259_curly_unconscious: SafeNPCRect<2>,

    #[serde(default = "default_n260_shovel_brigade_caged")]
    pub n260_shovel_brigade_caged: SafeNPCRect<6>,

    #[serde(default = "default_n261_chie_caged")]
    pub n261_chie_caged: SafeNPCRect<4>,

    #[serde(default = "default_n262_chaco_caged")]
    pub n262_chaco_caged: SafeNPCRect<4>,

    #[serde(default = "default_n263_doctor_boss")]
    pub n263_doctor_boss: SafeNPCRect<18>,

    #[serde(default = "default_n264_doctor_boss_red_projectile")]
    pub n264_doctor_boss_red_projectile: Rect<u16>,

    #[serde(default = "default_n265_doctor_boss_red_projectile_trail")]
    pub n265_doctor_boss_red_projectile_trail: SafeNPCRect<3>,

    #[serde(default = "default_n266_doctor_boss_red_projectile_bouncing")]
    pub n266_doctor_boss_red_projectile_bouncing: SafeNPCRect<2>,

    #[serde(default = "default_n267_muscle_doctor")]
    pub n267_muscle_doctor: SafeNPCRect<20>,

    #[serde(default = "default_n268_igor_enemy")]
    pub n268_igor_enemy: SafeNPCRect<20>,

    #[serde(default = "default_n269_red_bat_bouncing")]
    pub n269_red_bat_bouncing: SafeNPCRect<6>,

    #[serde(default = "default_n270_doctor_red_energy")]
    pub n270_doctor_red_energy: SafeNPCRect<2>,

    // pub n271_ironhead_block: () // Defined in code

    // pub n272_ironhead_block_generator: () // Defined in code
    #[serde(default = "default_n273_droll_projectile")]
    pub n273_droll_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n274_droll")]
    pub n274_droll: SafeNPCRect<12>,

    #[serde(default = "default_n275_puppy_plantation")]
    pub n275_puppy_plantation: SafeNPCRect<4>,

    #[serde(default = "default_n276_red_demon")]
    pub n276_red_demon: SafeNPCRect<18>,

    #[serde(default = "default_n277_red_demon_projectile")]
    pub n277_red_demon_projectile: SafeNPCRect<3>,

    #[serde(default = "default_n278_little_family")]
    pub n278_little_family: SafeNPCRect<6>,

    #[serde(default = "default_n279_large_falling_block")]
    pub n279_large_falling_block: SafeNPCRect<2>,

    #[serde(default = "default_n280_sue_teleported")]
    pub n280_sue_teleported: SafeNPCRect<4>,

    // pub n281_doctor_energy_form: () // Defined in code
    #[serde(default = "default_n282_mini_undead_core_active")]
    pub n282_mini_undead_core_active: SafeNPCRect<3>,

    #[serde(default = "default_n283_misery_possessed")]
    pub n283_misery_possessed: SafeNPCRect<22>,

    #[serde(default = "default_n284_sue_possessed")]
    pub n284_sue_possessed: SafeNPCRect<26>,

    #[serde(default = "default_n285_undead_core_spiral_projectile")]
    pub n285_undead_core_spiral_projectile: Rect<u16>,

    #[serde(default = "default_n286_undead_core_spiral_projectile_trail")]
    pub n286_undead_core_spiral_projectile_trail: SafeNPCRect<3>,

    #[serde(default = "default_n287_orange_smoke")]
    pub n287_orange_smoke: SafeNPCRect<7>,

    #[serde(default = "default_n288_undead_core_exploding_rock")]
    pub n288_undead_core_exploding_rock: SafeNPCRect<5>,

    #[serde(default = "default_n289_critter_orange")]
    pub n289_critter_orange: SafeNPCRect<6>,

    #[serde(default = "default_n290_bat_misery")]
    pub n290_bat_misery: SafeNPCRect<6>,

    #[serde(default = "default_n291_mini_undead_core_inactive")]
    pub n291_mini_undead_core_inactive: SafeNPCRect<2>,

    // pub n292_quake: () // Defined in code
    #[serde(default = "default_n293_undead_core_energy_shot")]
    pub n293_undead_core_energy_shot: SafeNPCRect<2>,

    // pub n294_quake_falling_block_generator: () // Defined in code
    #[serde(default = "default_n295_cloud")]
    pub n295_cloud: SafeNPCRect<4>,

    // pub n296_cloud_generator: () // Defined in code
    #[serde(default = "default_n297_sue_dragon_mouth")]
    pub n297_sue_dragon_mouth: Rect<u16>,

    #[serde(default = "default_n298_intro_doctor")]
    pub n298_intro_doctor: SafeNPCRect<8>,

    #[serde(default = "default_n299_intro_balrog_misery")]
    pub n299_intro_balrog_misery: SafeNPCRect<2>,

    #[serde(default = "default_n300_intro_demon_crown")]
    pub n300_intro_demon_crown: Rect<u16>,

    #[serde(default = "default_n301_misery_fish_missile")]
    pub n301_misery_fish_missile: SafeNPCRect<8>,

    // pub n302_camera_focus_marker: () // Defined in code
    #[serde(default = "default_n303_curly_machine_gun")]
    pub n303_curly_machine_gun: SafeNPCRect<4>,

    #[serde(default = "default_n304_gaudi_hospital")]
    pub n304_gaudi_hospital: SafeNPCRect<4>,

    #[serde(default = "default_n305_small_puppy")]
    pub n305_small_puppy: SafeNPCRect<4>,

    #[serde(default = "default_n306_balrog_nurse")]
    pub n306_balrog_nurse: SafeNPCRect<4>,

    #[serde(default = "default_n307_santa_caged")]
    pub n307_santa_caged: SafeNPCRect<4>,

    #[serde(default = "default_n308_stumpy")]
    pub n308_stumpy: SafeNPCRect<4>,

    #[serde(default = "default_n309_bute")]
    pub n309_bute: SafeNPCRect<4>,

    #[serde(default = "default_n310_bute_sword")]
    pub n310_bute_sword: SafeNPCRect<10>,

    #[serde(default = "default_n311_bute_archer")]
    pub n311_bute_archer: SafeNPCRect<14>,

    #[serde(default = "default_n312_bute_arrow_projectile")]
    pub n312_bute_arrow_projectile: SafeNPCRect<10>,

    #[serde(default = "default_n313_ma_pignon")]
    pub n313_ma_pignon: SafeNPCRect<28>,

    #[serde(default = "default_n314_ma_pignon_rock")]
    pub n314_ma_pignon_rock: SafeNPCRect<3>,

    #[serde(default = "default_n315_ma_pignon_clone")]
    pub n315_ma_pignon_clone: SafeNPCRect<8>,

    #[serde(default = "default_n316_bute_dead")]
    pub n316_bute_dead: SafeNPCRect<6>,

    #[serde(default = "default_n317_mesa")]
    pub n317_mesa: SafeNPCRect<8>,

    #[serde(default = "default_n318_mesa_dead")]
    pub n318_mesa_dead: SafeNPCRect<6>,

    #[serde(default = "default_n319_mesa_block")]
    pub n319_mesa_block: SafeNPCRect<3>,

    #[serde(default = "default_n320_curly_carried")]
    pub n320_curly_carried: SafeNPCRect<6>,

    #[serde(default = "default_n321_curly_nemesis")]
    pub n321_curly_nemesis: SafeNPCRect<6>,

    #[serde(default = "default_n322_deleet")]
    pub n322_deleet: SafeNPCRect<3>,

    #[serde(default = "default_n323_bute_spinning")]
    pub n323_bute_spinning: SafeNPCRect<4>,

    // pub n324_bute_generator: () // Defined in code
    #[serde(default = "default_n325_heavy_press_lightning")]
    pub n325_heavy_press_lightning: SafeNPCRect<7>,

    #[serde(default = "default_n326_sue_itoh_human_transition")]
    pub n326_sue_itoh_human_transition: SafeNPCRect<16>,

    #[serde(default = "default_n327_sneeze")]
    pub n327_sneeze: SafeNPCRect<2>,

    #[serde(default = "default_n328_human_transform_machine")]
    pub n328_human_transform_machine: Rect<u16>,

    #[serde(default = "default_n329_laboratory_fan")]
    pub n329_laboratory_fan: SafeNPCRect<2>,

    #[serde(default = "default_n330_rolling")]
    pub n330_rolling: SafeNPCRect<3>,

    #[serde(default = "default_n331_ballos_bone_projectile")]
    pub n331_ballos_bone_projectile: SafeNPCRect<4>,

    #[serde(default = "default_n332_ballos_shockwave")]
    pub n332_ballos_shockwave: SafeNPCRect<3>,

    #[serde(default = "default_n333_ballos_lightning")]
    pub n333_ballos_lightning: SafeNPCRect<2>,

    #[serde(default = "default_n334_sweat")]
    pub n334_sweat: SafeNPCRect<4>,

    #[serde(default = "default_n335_ikachan")]
    pub n335_ikachan: SafeNPCRect<3>,

    // pub n336_ikachan_generator: () // Defined in code
    #[serde(default = "default_n337_numahachi")]
    pub n337_numahachi: SafeNPCRect<2>,

    #[serde(default = "default_n338_green_devil")]
    pub n338_green_devil: SafeNPCRect<4>,

    // pub n339_green_devil_generator: () // Defined in code
    #[serde(default = "default_n340_ballos")]
    pub n340_ballos: SafeNPCRect<22>,

    #[serde(default = "default_n341_ballos_1_head")]
    pub n341_ballos_1_head: SafeNPCRect<3>,

    #[serde(default = "default_n342_ballos_orbiting_eye")]
    pub n342_ballos_orbiting_eye: SafeNPCRect<3>,

    #[serde(default = "default_n343_ballos_3_cutscene")]
    pub n343_ballos_3_cutscene: Rect<u16>,

    #[serde(default = "default_n344_ballos_3_eyes")]
    pub n344_ballos_3_eyes: SafeNPCRect<2>,

    #[serde(default = "default_n345_ballos_skull_projectile")]
    pub n345_ballos_skull_projectile: SafeNPCRect<4>,

    #[serde(default = "default_n346_ballos_orbiting_platform")]
    pub n346_ballos_orbiting_platform: Rect<u16>,

    #[serde(default = "default_n347_hoppy")]
    pub n347_hoppy: SafeNPCRect<4>,

    #[serde(default = "default_n348_ballos_4_spikes")]
    pub n348_ballos_4_spikes: SafeNPCRect<2>,

    #[serde(default = "default_n349_statue")]
    pub n349_statue: Rect<u16>,

    #[serde(default = "default_n350_flying_bute_archer")]
    pub n350_flying_bute_archer: SafeNPCRect<14>,

    #[serde(default = "default_n351_statue_shootable")]
    pub n351_statue_shootable: SafeNPCRect<9>,

    #[serde(default = "default_n352_ending_characters")]
    pub n352_ending_characters: SafeNPCRect<28>,

    #[serde(default = "default_n353_bute_sword_flying")]
    pub n353_bute_sword_flying: SafeNPCRect<8>,

    // pub n354_invisible_deathtrap_wall: () // Defined in code
    #[serde(default = "default_n355_quote_and_curly_on_balrog")]
    pub n355_quote_and_curly_on_balrog: SafeNPCRect<4>,

    #[serde(default = "default_n356_balrog_rescuing")]
    pub n356_balrog_rescuing: SafeNPCRect<2>,

    #[serde(default = "default_n357_puppy_ghost")]
    pub n357_puppy_ghost: Rect<u16>,

    #[serde(default = "default_n358_misery_credits")]
    pub n358_misery_credits: SafeNPCRect<5>,

    // pub n359_water_droplet_generator: () // Defined in code
    #[serde(default = "default_n360_credits_thank_you")]
    pub n360_credits_thank_you: Rect<u16>,

    // pub n370_second_quote: () // Same as n150_quote
    #[serde(default = "default_b01_omega")]
    pub b01_omega: SafeNPCRect<10>,

    #[serde(default = "default_b02_balfrog")]
    pub b02_balfrog: SafeNPCRect<18>,

    #[serde(default = "default_b03_monster_x")]
    pub b03_monster_x: SafeNPCRect<29>,

    #[serde(default = "default_b04_core")]
    pub b04_core: SafeNPCRect<10>,

    #[serde(default = "default_b05_ironhead")]
    pub b05_ironhead: SafeNPCRect<18>,

    #[serde(default = "default_b06_sisters")]
    pub b06_sisters: SafeNPCRect<14>,

    #[serde(default = "default_b07_undead_core")]
    pub b07_undead_core: SafeNPCRect<15>,

    #[serde(default = "default_b08_heavy_press")]
    pub b08_heavy_press: SafeNPCRect<6>,

    #[serde(default = "default_b09_ballos")]
    pub b09_ballos: SafeNPCRect<14>,
}

fn default_n001_experience() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 64, top: 16, right: 80, bottom: 32 },
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
    ])
}

fn default_n002_behemoth() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 32, top: 0, right: 64, bottom: 24 },
        Rect { left: 0, top: 0, right: 32, bottom: 24 },
        Rect { left: 32, top: 0, right: 64, bottom: 24 },
        Rect { left: 64, top: 0, right: 96, bottom: 24 },
        Rect { left: 96, top: 0, right: 128, bottom: 24 },
        Rect { left: 128, top: 0, right: 160, bottom: 24 },
        Rect { left: 160, top: 0, right: 192, bottom: 24 },
        Rect { left: 32, top: 24, right: 64, bottom: 48 },
        Rect { left: 0, top: 24, right: 32, bottom: 48 },
        Rect { left: 32, top: 24, right: 64, bottom: 48 },
        Rect { left: 64, top: 24, right: 96, bottom: 48 },
        Rect { left: 96, top: 24, right: 128, bottom: 48 },
        Rect { left: 128, top: 24, right: 160, bottom: 48 },
        Rect { left: 160, top: 24, right: 192, bottom: 48 },
    ])
}

fn default_n004_smoke() -> SafeNPCRect<16> {
    SafeNPCRect([
        Rect { left: 16, top: 0, right: 17, bottom: 1 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 64, top: 0, right: 80, bottom: 16 },
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 16, top: 0, right: 17, bottom: 1 },
        Rect { left: 80, top: 48, right: 96, bottom: 64 },
        Rect { left: 0, top: 128, right: 16, bottom: 144 },
        Rect { left: 16, top: 128, right: 32, bottom: 144 },
        Rect { left: 32, top: 128, right: 48, bottom: 144 },
        Rect { left: 48, top: 128, right: 64, bottom: 144 },
        Rect { left: 64, top: 128, right: 80, bottom: 144 },
        Rect { left: 80, top: 128, right: 96, bottom: 144 },
    ])
}

fn default_n005_green_critter() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 16, top: 48, right: 32, bottom: 64 },
        Rect { left: 32, top: 48, right: 48, bottom: 64 },
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 32, top: 64, right: 48, bottom: 80 },
    ])
}

fn default_n006_green_beetle() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 48, top: 80, right: 64, bottom: 96 },
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 16, top: 96, right: 32, bottom: 112 },
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 64, top: 96, right: 80, bottom: 112 },
    ])
}

fn default_n007_basil() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 256, top: 64, right: 288, bottom: 80 },
        Rect { left: 256, top: 80, right: 288, bottom: 96 },
        Rect { left: 256, top: 96, right: 288, bottom: 112 },
        Rect { left: 288, top: 64, right: 320, bottom: 80 },
        Rect { left: 288, top: 80, right: 320, bottom: 96 },
        Rect { left: 288, top: 96, right: 320, bottom: 112 },
    ])
}

fn default_n008_blue_beetle() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 80, top: 80, right: 96, bottom: 96 },
        Rect { left: 96, top: 80, right: 112, bottom: 96 },
        Rect { left: 80, top: 96, right: 96, bottom: 112 },
        Rect { left: 96, top: 96, right: 112, bottom: 112 },
    ])
}

fn default_n009_balrog_falling_in() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
    ])
}

fn default_n010_balrog_shooting() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 40, top: 0, right: 80, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 40, top: 24, right: 80, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
    ])
}

fn default_n011_balrog_energy_shot() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 208, top: 104, right: 224, bottom: 120 },
        Rect { left: 224, top: 104, right: 240, bottom: 120 },
        Rect { left: 240, top: 104, right: 256, bottom: 120 },
    ])
}

fn default_n012_balrog_cutscene() -> SafeNPCRect<28> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 160, top: 0, right: 200, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 240, top: 0, right: 280, bottom: 24 },
        Rect { left: 200, top: 0, right: 240, bottom: 24 },
        Rect { left: 280, top: 0, right: 320, bottom: 24 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 80, top: 48, right: 120, bottom: 72 },
        Rect { left: 0, top: 48, right: 40, bottom: 72 },
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 40, top: 48, right: 80, bottom: 72 },
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 280, top: 0, right: 320, bottom: 24 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 160, top: 24, right: 200, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
        Rect { left: 240, top: 24, right: 280, bottom: 48 },
        Rect { left: 200, top: 24, right: 240, bottom: 48 },
        Rect { left: 280, top: 24, right: 320, bottom: 48 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 80, top: 72, right: 120, bottom: 96 },
        Rect { left: 0, top: 72, right: 40, bottom: 96 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 40, top: 72, right: 80, bottom: 96 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 280, top: 24, right: 320, bottom: 48 },
    ])
}

fn default_n013_forcefield() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
    ])
}

fn default_n014_key() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 192, top: 0, right: 208, bottom: 16 },
        Rect { left: 208, top: 0, right: 224, bottom: 16 },
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
    ])
}

fn default_n015_closed_chest() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 240, top: 0, right: 256, bottom: 16 },
        Rect { left: 256, top: 0, right: 272, bottom: 16 },
        Rect { left: 272, top: 0, right: 288, bottom: 16 },
    ])
}

fn default_n016_save_point() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 192, top: 16, right: 208, bottom: 32 },
        Rect { left: 208, top: 16, right: 224, bottom: 32 },
    ])
}

fn default_n017_health_refill() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 288, top: 0, right: 304, bottom: 16 },
        Rect { left: 304, top: 0, right: 320, bottom: 16 },
    ])
}

fn default_n018_door() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 224, top: 16, right: 240, bottom: 40 },
        Rect { left: 192, top: 112, right: 208, bottom: 136 },
    ])
}

fn default_n019_balrog_bust_in() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 160, top: 0, right: 200, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 160, top: 24, right: 200, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
    ])
}

fn default_n020_computer() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 288, top: 16, right: 320, bottom: 40 },
        Rect { left: 288, top: 40, right: 320, bottom: 64 },
        Rect { left: 288, top: 40, right: 320, bottom: 64 },
        Rect { left: 288, top: 64, right: 320, bottom: 88 },
    ])
}

fn default_n021_chest_open() -> Rect<u16> {
    Rect { left: 224, top: 40, right: 240, bottom: 48 }
}

fn default_n022_teleporter() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 240, top: 16, right: 264, bottom: 48 },
        Rect { left: 248, top: 152, right: 272, bottom: 184 },
    ])
}

fn default_n023_teleporter_lights() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 264, top: 16, right: 288, bottom: 20 },
        Rect { left: 264, top: 20, right: 288, bottom: 24 },
        Rect { left: 264, top: 24, right: 288, bottom: 28 },
        Rect { left: 264, top: 28, right: 288, bottom: 32 },
        Rect { left: 264, top: 32, right: 288, bottom: 36 },
        Rect { left: 264, top: 36, right: 288, bottom: 40 },
        Rect { left: 264, top: 40, right: 288, bottom: 44 },
        Rect { left: 264, top: 44, right: 288, bottom: 48 },
    ])
}

fn default_n024_power_critter() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 24, bottom: 24 },
        Rect { left: 24, top: 0, right: 48, bottom: 24 },
        Rect { left: 48, top: 0, right: 72, bottom: 24 },
        Rect { left: 72, top: 0, right: 96, bottom: 24 },
        Rect { left: 96, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 144, bottom: 24 },
        Rect { left: 0, top: 24, right: 24, bottom: 48 },
        Rect { left: 24, top: 24, right: 48, bottom: 48 },
        Rect { left: 48, top: 24, right: 72, bottom: 48 },
        Rect { left: 72, top: 24, right: 96, bottom: 48 },
        Rect { left: 96, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 144, bottom: 48 },
    ])
}

fn default_n025_lift() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 256, top: 64, right: 288, bottom: 80 },
        Rect { left: 256, top: 80, right: 288, bottom: 96 },
    ])
}

fn default_n026_bat_flying() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 48, top: 80, right: 64, bottom: 96 },
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 80, top: 80, right: 96, bottom: 96 },
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 64, top: 96, right: 80, bottom: 112 },
        Rect { left: 80, top: 96, right: 96, bottom: 112 },
    ])
}

fn default_n027_death_trap() -> Rect<u16> {
    Rect { left: 96, top: 64, right: 128, bottom: 88 }
}

fn default_n028_flying_critter() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 16, top: 48, right: 32, bottom: 64 },
        Rect { left: 32, top: 48, right: 48, bottom: 64 },
        Rect { left: 48, top: 48, right: 64, bottom: 64 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
        Rect { left: 80, top: 48, right: 96, bottom: 64 },
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 32, top: 64, right: 48, bottom: 80 },
        Rect { left: 48, top: 64, right: 64, bottom: 80 },
        Rect { left: 64, top: 64, right: 80, bottom: 80 },
        Rect { left: 80, top: 64, right: 96, bottom: 80 },
    ])
}

fn default_n029_cthulhu() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 192, right: 16, bottom: 216 },
        Rect { left: 16, top: 192, right: 32, bottom: 216 },
        Rect { left: 0, top: 216, right: 16, bottom: 240 },
        Rect { left: 16, top: 216, right: 32, bottom: 240 },
    ])
}

fn default_n030_hermit_gunsmith() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 0, top: 32, right: 16, bottom: 48 },
    ])
}

fn default_n031_bat_hanging() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 48, top: 80, right: 64, bottom: 96 },
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 16, top: 96, right: 32, bottom: 112 },
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 64, top: 96, right: 80, bottom: 112 },
    ])
}

fn default_n032_life_capsule() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
    ])
}

fn default_n033_balrog_bouncing_projectile() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 240, top: 64, right: 256, bottom: 80 },
        Rect { left: 240, top: 80, right: 256, bottom: 96 },
    ])
}

fn default_n034_bed() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 192, top: 48, right: 224, bottom: 64 },
        Rect { left: 192, top: 184, right: 224, bottom: 200 },
    ])
}

fn default_n035_mannan() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 96, top: 64, right: 120, bottom: 96 },
        Rect { left: 120, top: 64, right: 144, bottom: 96 },
        Rect { left: 144, top: 64, right: 168, bottom: 96 },
        Rect { left: 168, top: 64, right: 192, bottom: 96 },
        Rect { left: 96, top: 96, right: 120, bottom: 128 },
        Rect { left: 120, top: 96, right: 144, bottom: 128 },
        Rect { left: 144, top: 96, right: 168, bottom: 128 },
        Rect { left: 168, top: 96, right: 192, bottom: 128 },
    ])
}

fn default_n036_balrog_hover() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 40, top: 0, right: 80, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 160, top: 48, right: 200, bottom: 72 },
        Rect { left: 200, top: 48, right: 240, bottom: 72 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 40, top: 24, right: 80, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
        Rect { left: 160, top: 72, right: 200, bottom: 96 },
        Rect { left: 200, top: 72, right: 240, bottom: 96 },
    ])
}

fn default_n037_sign() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 192, top: 64, right: 208, bottom: 80 },
        Rect { left: 208, top: 64, right: 224, bottom: 80 },
    ])
}

fn default_n038_fireplace() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 128, top: 64, right: 144, bottom: 80 },
        Rect { left: 144, top: 64, right: 160, bottom: 80 },
        Rect { left: 160, top: 64, right: 176, bottom: 80 },
        Rect { left: 176, top: 64, right: 192, bottom: 80 },
    ])
}

fn default_n039_save_sign() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 224, top: 64, right: 240, bottom: 80 },
        Rect { left: 240, top: 64, right: 256, bottom: 80 },
    ])
}

fn default_n040_santa() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 0, top: 32, right: 16, bottom: 48 },
        Rect { left: 16, top: 32, right: 32, bottom: 48 },
        Rect { left: 32, top: 32, right: 48, bottom: 48 },
        Rect { left: 0, top: 32, right: 16, bottom: 48 },
        Rect { left: 48, top: 32, right: 64, bottom: 48 },
        Rect { left: 0, top: 32, right: 16, bottom: 48 },
        Rect { left: 64, top: 32, right: 80, bottom: 48 },
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 16, top: 48, right: 32, bottom: 64 },
        Rect { left: 32, top: 48, right: 48, bottom: 64 },
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 48, top: 48, right: 64, bottom: 64 },
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
    ])
}

fn default_n041_busted_door() -> Rect<u16> {
    Rect { left: 0, top: 80, right: 48, bottom: 112 }
}

fn default_n042_sue() -> SafeNPCRect<26> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 64, top: 0, right: 80, bottom: 16 },
        Rect { left: 80, top: 32, right: 96, bottom: 48 },
        Rect { left: 96, top: 32, right: 112, bottom: 48 },
        Rect { left: 128, top: 32, right: 144, bottom: 48 },
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
        Rect { left: 160, top: 32, right: 176, bottom: 48 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 64, top: 16, right: 80, bottom: 32 },
        Rect { left: 80, top: 48, right: 96, bottom: 64 },
        Rect { left: 96, top: 48, right: 112, bottom: 64 },
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 160, top: 48, right: 176, bottom: 64 },
    ])
}

fn default_n043_chalkboard() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 128, top: 80, right: 168, bottom: 112 },
        Rect { left: 168, top: 80, right: 208, bottom: 112 },
    ])
}

fn default_n044_polish() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 32, bottom: 32 },
        Rect { left: 96, top: 0, right: 128, bottom: 32 },
        Rect { left: 128, top: 0, right: 160, bottom: 32 },
        Rect { left: 0, top: 0, right: 32, bottom: 32 },
        Rect { left: 32, top: 0, right: 64, bottom: 32 },
        Rect { left: 64, top: 0, right: 96, bottom: 32 },
    ])
}

fn default_n045_baby() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 32, right: 16, bottom: 48 },
        Rect { left: 16, top: 32, right: 32, bottom: 48 },
        Rect { left: 32, top: 32, right: 48, bottom: 48 },
    ])
}

fn default_n047_sandcroc() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 0, top: 48, right: 48, bottom: 80 },
        Rect { left: 48, top: 48, right: 96, bottom: 80 },
        Rect { left: 96, top: 48, right: 144, bottom: 80 },
        Rect { left: 144, top: 48, right: 192, bottom: 80 },
        Rect { left: 192, top: 48, right: 240, bottom: 80 },
    ])
}

fn default_n048_omega_projectiles() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 288, top: 88, right: 304, bottom: 104 },
        Rect { left: 304, top: 88, right: 320, bottom: 104 },
        Rect { left: 288, top: 104, right: 304, bottom: 120 },
        Rect { left: 304, top: 104, right: 320, bottom: 120 },
    ])
}

fn default_n049_skullhead() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 80, right: 32, bottom: 104 },
        Rect { left: 32, top: 80, right: 64, bottom: 104 },
        Rect { left: 64, top: 80, right: 96, bottom: 104 },
        Rect { left: 0, top: 104, right: 32, bottom: 128 },
        Rect { left: 32, top: 104, right: 64, bottom: 128 },
        Rect { left: 64, top: 104, right: 96, bottom: 128 },
    ])
}

fn default_n050_skeleton_projectile() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 48, top: 32, right: 64, bottom: 48 },
        Rect { left: 64, top: 32, right: 80, bottom: 48 },
        Rect { left: 80, top: 32, right: 96, bottom: 48 },
        Rect { left: 96, top: 32, right: 112, bottom: 48 },
    ])
}

fn default_n051_crow_and_skullhead() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 96, top: 80, right: 128, bottom: 112 },
        Rect { left: 128, top: 80, right: 160, bottom: 112 },
        Rect { left: 160, top: 80, right: 192, bottom: 112 },
        Rect { left: 192, top: 80, right: 224, bottom: 112 },
        Rect { left: 224, top: 80, right: 256, bottom: 112 },
        Rect { left: 96, top: 112, right: 128, bottom: 144 },
        Rect { left: 128, top: 112, right: 160, bottom: 144 },
        Rect { left: 160, top: 112, right: 192, bottom: 144 },
        Rect { left: 192, top: 112, right: 224, bottom: 144 },
        Rect { left: 224, top: 112, right: 256, bottom: 144 },
    ])
}

fn default_n052_sitting_blue_robot() -> Rect<u16> {
    Rect { left: 240, top: 96, right: 256, bottom: 112 }
}

fn default_n053_skullstep_leg() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 128, right: 24, bottom: 144 },
        Rect { left: 24, top: 128, right: 48, bottom: 144 },
        Rect { left: 48, top: 128, right: 72, bottom: 144 },
        Rect { left: 72, top: 128, right: 96, bottom: 144 },
    ])
}

fn default_n054_skullstep() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 80, right: 32, bottom: 104 },
        Rect { left: 32, top: 80, right: 64, bottom: 104 },
        Rect { left: 64, top: 80, right: 96, bottom: 104 },
        Rect { left: 0, top: 104, right: 32, bottom: 128 },
        Rect { left: 32, top: 104, right: 64, bottom: 128 },
        Rect { left: 64, top: 104, right: 96, bottom: 128 },
    ])
}

fn default_n055_kazuma() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 192, top: 192, right: 208, bottom: 216 },
        Rect { left: 208, top: 192, right: 224, bottom: 216 },
        Rect { left: 192, top: 192, right: 208, bottom: 216 },
        Rect { left: 224, top: 192, right: 240, bottom: 216 },
        Rect { left: 192, top: 192, right: 208, bottom: 216 },
        Rect { left: 240, top: 192, right: 256, bottom: 216 },
        Rect { left: 192, top: 216, right: 208, bottom: 240 },
        Rect { left: 208, top: 216, right: 224, bottom: 240 },
        Rect { left: 192, top: 216, right: 208, bottom: 240 },
        Rect { left: 224, top: 216, right: 240, bottom: 240 },
        Rect { left: 192, top: 216, right: 208, bottom: 240 },
        Rect { left: 240, top: 216, right: 256, bottom: 240 },
    ])
}

fn default_n056_tan_beetle() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 144, right: 16, bottom: 160 },
        Rect { left: 16, top: 144, right: 32, bottom: 160 },
        Rect { left: 32, top: 144, right: 48, bottom: 160 },
        Rect { left: 0, top: 160, right: 16, bottom: 176 },
        Rect { left: 16, top: 160, right: 32, bottom: 176 },
        Rect { left: 32, top: 160, right: 48, bottom: 176 },
    ])
}

fn default_n057_crow() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 96, top: 80, right: 128, bottom: 112 },
        Rect { left: 128, top: 80, right: 160, bottom: 112 },
        Rect { left: 160, top: 80, right: 192, bottom: 112 },
        Rect { left: 192, top: 80, right: 224, bottom: 112 },
        Rect { left: 224, top: 80, right: 256, bottom: 112 },
        Rect { left: 96, top: 112, right: 128, bottom: 144 },
        Rect { left: 128, top: 112, right: 160, bottom: 144 },
        Rect { left: 160, top: 112, right: 192, bottom: 144 },
        Rect { left: 192, top: 112, right: 224, bottom: 144 },
        Rect { left: 224, top: 112, right: 256, bottom: 144 },
    ])
}

fn default_n058_basu() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 192, top: 0, right: 216, bottom: 24 },
        Rect { left: 216, top: 0, right: 240, bottom: 24 },
        Rect { left: 240, top: 0, right: 264, bottom: 24 },
        Rect { left: 192, top: 24, right: 216, bottom: 48 },
        Rect { left: 216, top: 24, right: 240, bottom: 48 },
        Rect { left: 240, top: 24, right: 264, bottom: 48 },
    ])
}

fn default_n059_eye_door() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 224, top: 16, right: 240, bottom: 40 },
        Rect { left: 208, top: 80, right: 224, bottom: 104 },
        Rect { left: 224, top: 80, right: 240, bottom: 104 },
        Rect { left: 240, top: 80, right: 256, bottom: 104 },
    ])
}

fn default_n060_toroko() -> SafeNPCRect<16> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 32, top: 64, right: 48, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 48, top: 64, right: 64, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 112, top: 64, right: 128, bottom: 80 },
        Rect { left: 128, top: 64, right: 144, bottom: 80 },
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 48, top: 80, right: 64, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 112, top: 80, right: 128, bottom: 96 },
        Rect { left: 128, top: 80, right: 144, bottom: 96 },
    ])
}

fn default_n061_king() -> SafeNPCRect<22> {
    SafeNPCRect([
        Rect { left: 224, top: 32, right: 240, bottom: 48 },
        Rect { left: 240, top: 32, right: 256, bottom: 48 },
        Rect { left: 256, top: 32, right: 272, bottom: 48 },
        Rect { left: 272, top: 32, right: 288, bottom: 48 },
        Rect { left: 288, top: 32, right: 304, bottom: 48 },
        Rect { left: 224, top: 32, right: 240, bottom: 48 },
        Rect { left: 304, top: 32, right: 320, bottom: 48 },
        Rect { left: 224, top: 32, right: 240, bottom: 48 },
        Rect { left: 272, top: 32, right: 288, bottom: 48 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
        Rect { left: 224, top: 48, right: 240, bottom: 64 },
        Rect { left: 240, top: 48, right: 256, bottom: 64 },
        Rect { left: 256, top: 48, right: 272, bottom: 64 },
        Rect { left: 272, top: 48, right: 288, bottom: 64 },
        Rect { left: 288, top: 48, right: 304, bottom: 64 },
        Rect { left: 224, top: 48, right: 240, bottom: 64 },
        Rect { left: 304, top: 48, right: 320, bottom: 64 },
        Rect { left: 224, top: 48, right: 240, bottom: 64 },
        Rect { left: 272, top: 48, right: 288, bottom: 64 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
    ])
}

fn default_n062_kazuma_computer() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 272, top: 192, right: 288, bottom: 216 },
        Rect { left: 288, top: 192, right: 304, bottom: 216 },
        Rect { left: 304, top: 192, right: 320, bottom: 216 },
    ])
}

fn default_n063_toroko_stick() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 64, top: 64, right: 80, bottom: 80 },
        Rect { left: 80, top: 64, right: 96, bottom: 80 },
        Rect { left: 64, top: 64, right: 80, bottom: 80 },
        Rect { left: 96, top: 64, right: 112, bottom: 80 },
        Rect { left: 112, top: 64, right: 128, bottom: 80 },
        Rect { left: 128, top: 64, right: 144, bottom: 80 },
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 80, top: 80, right: 96, bottom: 96 },
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 96, top: 80, right: 112, bottom: 96 },
        Rect { left: 112, top: 80, right: 128, bottom: 96 },
        Rect { left: 128, top: 80, right: 144, bottom: 96 },
    ])
}

fn default_n064_first_cave_critter() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
    ])
}

fn default_n065_first_cave_bat() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 32, top: 32, right: 48, bottom: 48 },
        Rect { left: 48, top: 32, right: 64, bottom: 48 },
        Rect { left: 64, top: 32, right: 80, bottom: 48 },
        Rect { left: 80, top: 32, right: 96, bottom: 48 },
        Rect { left: 32, top: 48, right: 48, bottom: 64 },
        Rect { left: 48, top: 48, right: 64, bottom: 64 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
        Rect { left: 80, top: 48, right: 96, bottom: 64 },
    ])
}

fn default_n066_misery_bubble() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 32, top: 192, right: 56, bottom: 216 },
        Rect { left: 56, top: 192, right: 80, bottom: 216 },
        Rect { left: 32, top: 216, right: 56, bottom: 240 },
        Rect { left: 56, top: 216, right: 80, bottom: 240 },
    ])
}

fn default_n067_misery_floating() -> SafeNPCRect<16> {
    SafeNPCRect([
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
    ])
}

fn default_n068_balrog_running() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 0, top: 48, right: 40, bottom: 72 },
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 40, top: 48, right: 80, bottom: 72 },
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 80, top: 48, right: 120, bottom: 72 },
        Rect { left: 120, top: 48, right: 160, bottom: 72 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 0, top: 72, right: 40, bottom: 96 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 40, top: 72, right: 80, bottom: 96 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 80, top: 72, right: 120, bottom: 96 },
        Rect { left: 120, top: 72, right: 160, bottom: 96 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
    ])
}

fn default_n069_pignon() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 64, top: 0, right: 80, bottom: 16 },
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 64, top: 16, right: 80, bottom: 32 },
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
    ])
}

fn default_n070_sparkle() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 96, top: 48, right: 112, bottom: 64 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
        Rect { left: 144, top: 48, right: 160, bottom: 64 },
    ])
}

fn default_n071_chinfish() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 64, top: 32, right: 80, bottom: 48 },
        Rect { left: 80, top: 32, right: 96, bottom: 48 },
        Rect { left: 96, top: 32, right: 112, bottom: 48 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
        Rect { left: 80, top: 48, right: 96, bottom: 64 },
        Rect { left: 96, top: 48, right: 112, bottom: 64 },
    ])
}

fn default_n072_sprinkler() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 224, top: 48, right: 240, bottom: 64 },
        Rect { left: 240, top: 48, right: 256, bottom: 64 },
    ])
}

fn default_n073_water_droplet() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 72, top: 16, right: 74, bottom: 18 },
        Rect { left: 74, top: 16, right: 76, bottom: 18 },
        Rect { left: 76, top: 16, right: 78, bottom: 18 },
        Rect { left: 78, top: 16, right: 80, bottom: 18 },
        Rect { left: 80, top: 16, right: 82, bottom: 18 },
    ])
}

fn default_n074_jack() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 64, top: 0, right: 80, bottom: 16 },
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 64, top: 0, right: 80, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 64, top: 0, right: 80, bottom: 16 },
        Rect { left: 64, top: 16, right: 80, bottom: 32 },
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
        Rect { left: 64, top: 16, right: 80, bottom: 32 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
        Rect { left: 64, top: 16, right: 80, bottom: 32 },
    ])
}

fn default_n075_kanpachi() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 272, top: 32, right: 296, bottom: 56 },
        Rect { left: 296, top: 32, right: 320, bottom: 56 },
    ])
}

fn default_n077_yamashita() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 16, right: 48, bottom: 48 },
        Rect { left: 48, top: 16, right: 96, bottom: 48 },
        Rect { left: 96, top: 16, right: 144, bottom: 48 },
    ])
}

fn default_n078_pot() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 160, top: 48, right: 176, bottom: 64 },
        Rect { left: 176, top: 48, right: 192, bottom: 64 },
    ])
}

fn default_n079_mahin() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
    ])
}

fn default_n080_gravekeeper() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 24, bottom: 88 },
        Rect { left: 24, top: 64, right: 48, bottom: 88 },
        Rect { left: 0, top: 64, right: 24, bottom: 88 },
        Rect { left: 48, top: 64, right: 72, bottom: 88 },
        Rect { left: 72, top: 64, right: 96, bottom: 88 },
        Rect { left: 96, top: 64, right: 120, bottom: 88 },
        Rect { left: 120, top: 64, right: 144, bottom: 88 },
        Rect { left: 0, top: 88, right: 24, bottom: 112 },
        Rect { left: 24, top: 88, right: 48, bottom: 112 },
        Rect { left: 0, top: 88, right: 24, bottom: 112 },
        Rect { left: 48, top: 88, right: 72, bottom: 112 },
        Rect { left: 72, top: 88, right: 96, bottom: 112 },
        Rect { left: 96, top: 88, right: 120, bottom: 112 },
        Rect { left: 120, top: 88, right: 144, bottom: 112 },
    ])
}

fn default_n081_giant_pignon() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 144, top: 64, right: 168, bottom: 88 },
        Rect { left: 168, top: 64, right: 192, bottom: 88 },
        Rect { left: 192, top: 64, right: 216, bottom: 88 },
        Rect { left: 216, top: 64, right: 240, bottom: 88 },
        Rect { left: 144, top: 64, right: 168, bottom: 88 },
        Rect { left: 240, top: 64, right: 264, bottom: 88 },
        Rect { left: 144, top: 88, right: 168, bottom: 112 },
        Rect { left: 168, top: 88, right: 192, bottom: 112 },
        Rect { left: 192, top: 88, right: 216, bottom: 112 },
        Rect { left: 216, top: 88, right: 240, bottom: 112 },
        Rect { left: 144, top: 88, right: 168, bottom: 112 },
        Rect { left: 240, top: 88, right: 264, bottom: 112 },
    ])
}

fn default_n082_misery_standing() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 208, top: 64, right: 224, bottom: 80 },
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 208, top: 80, right: 224, bottom: 96 },
    ])
}

fn default_n083_igor_cutscene() -> SafeNPCRect<16> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 40, top: 0, right: 80, bottom: 40 },
        Rect { left: 80, top: 0, right: 120, bottom: 40 },
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 120, top: 0, right: 160, bottom: 40 },
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 160, top: 0, right: 200, bottom: 40 },
        Rect { left: 200, top: 0, right: 240, bottom: 40 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 40, top: 40, right: 80, bottom: 80 },
        Rect { left: 80, top: 40, right: 120, bottom: 80 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 120, top: 40, right: 160, bottom: 80 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 160, top: 40, right: 200, bottom: 80 },
        Rect { left: 200, top: 40, right: 240, bottom: 80 },
    ])
}

fn default_n084_basu_projectile() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 48, top: 48, right: 64, bottom: 64 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
        Rect { left: 48, top: 64, right: 64, bottom: 80 },
        Rect { left: 64, top: 64, right: 80, bottom: 80 },
    ])
}

fn default_n085_terminal() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 256, top: 96, right: 272, bottom: 120 },
        Rect { left: 256, top: 96, right: 272, bottom: 120 },
        Rect { left: 272, top: 96, right: 288, bottom: 120 },
        Rect { left: 256, top: 96, right: 272, bottom: 120 },
        Rect { left: 288, top: 96, right: 304, bottom: 120 },
        Rect { left: 304, top: 96, right: 320, bottom: 120 },
    ])
}

fn default_n086_missile_pickup() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 16, top: 112, right: 32, bottom: 128 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
    ])
}

fn default_n087_heart_pickup() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 48, top: 80, right: 64, bottom: 96 },
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 80, top: 80, right: 96, bottom: 96 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
    ])
}

fn default_n088_igor_boss() -> SafeNPCRect<24> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 40, top: 0, right: 80, bottom: 40 },
        Rect { left: 80, top: 0, right: 120, bottom: 40 },
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 120, top: 0, right: 160, bottom: 40 },
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 160, top: 0, right: 200, bottom: 40 },
        Rect { left: 200, top: 0, right: 240, bottom: 40 },
        Rect { left: 0, top: 80, right: 40, bottom: 120 },
        Rect { left: 40, top: 80, right: 80, bottom: 120 },
        Rect { left: 240, top: 0, right: 280, bottom: 40 },
        Rect { left: 280, top: 0, right: 320, bottom: 40 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 40, top: 40, right: 80, bottom: 80 },
        Rect { left: 80, top: 40, right: 120, bottom: 80 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 120, top: 40, right: 160, bottom: 80 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 160, top: 40, right: 200, bottom: 80 },
        Rect { left: 200, top: 40, right: 240, bottom: 80 },
        Rect { left: 120, top: 80, right: 160, bottom: 120 },
        Rect { left: 160, top: 80, right: 200, bottom: 120 },
        Rect { left: 240, top: 40, right: 280, bottom: 80 },
        Rect { left: 280, top: 40, right: 320, bottom: 80 },
    ])
}

fn default_n089_igor_dead() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 80, top: 80, right: 120, bottom: 120 },
        Rect { left: 240, top: 80, right: 264, bottom: 104 },
        Rect { left: 264, top: 80, right: 288, bottom: 104 },
        Rect { left: 288, top: 80, right: 312, bottom: 104 },
        Rect { left: 200, top: 80, right: 240, bottom: 120 },
        Rect { left: 240, top: 104, right: 264, bottom: 128 },
        Rect { left: 264, top: 104, right: 288, bottom: 128 },
        Rect { left: 288, top: 104, right: 312, bottom: 128 },
    ])
}

fn default_n090_background() -> Rect<u16> {
    Rect { left: 280, top: 80, right: 296, bottom: 104 }
}

fn default_n091_mimiga_cage() -> Rect<u16> {
    Rect { left: 96, top: 88, right: 128, bottom: 112 }
}

fn default_n092_sue_at_pc() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 272, top: 216, right: 288, bottom: 240 },
        Rect { left: 288, top: 216, right: 304, bottom: 240 },
        Rect { left: 304, top: 216, right: 320, bottom: 240 },
    ])
}

fn default_n093_chaco() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 32, top: 32, right: 48, bottom: 48 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 32, top: 32, right: 48, bottom: 48 },
    ])
}

fn default_n094_kulala() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 272, top: 0, right: 320, bottom: 24 },
        Rect { left: 272, top: 24, right: 320, bottom: 48 },
        Rect { left: 272, top: 48, right: 320, bottom: 72 },
        Rect { left: 272, top: 72, right: 320, bottom: 96 },
        Rect { left: 272, top: 96, right: 320, bottom: 120 },
    ])
}

fn default_n095_jelly() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 208, top: 64, right: 224, bottom: 80 },
        Rect { left: 224, top: 64, right: 240, bottom: 80 },
        Rect { left: 240, top: 64, right: 256, bottom: 80 },
        Rect { left: 256, top: 64, right: 272, bottom: 80 },
        Rect { left: 208, top: 80, right: 224, bottom: 96 },
        Rect { left: 224, top: 80, right: 240, bottom: 96 },
        Rect { left: 240, top: 80, right: 256, bottom: 96 },
        Rect { left: 256, top: 80, right: 272, bottom: 96 },
    ])
}

fn default_n096_fan_left() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 272, top: 120, right: 288, bottom: 136 },
        Rect { left: 288, top: 120, right: 304, bottom: 136 },
        Rect { left: 304, top: 120, right: 320, bottom: 136 },
    ])
}

fn default_n097_fan_up() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 272, top: 136, right: 288, bottom: 152 },
        Rect { left: 288, top: 136, right: 304, bottom: 152 },
        Rect { left: 304, top: 136, right: 320, bottom: 152 },
    ])
}

fn default_n098_fan_right() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 272, top: 152, right: 288, bottom: 168 },
        Rect { left: 288, top: 152, right: 304, bottom: 168 },
        Rect { left: 304, top: 152, right: 320, bottom: 168 },
    ])
}

fn default_n099_fan_down() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 272, top: 168, right: 288, bottom: 184 },
        Rect { left: 288, top: 168, right: 304, bottom: 184 },
        Rect { left: 304, top: 168, right: 320, bottom: 184 },
    ])
}

fn default_n100_grate() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 272, top: 48, right: 288, bottom: 64 },
        Rect { left: 272, top: 48, right: 288, bottom: 64 },
    ])
}

fn default_n101_malco_screen() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 240, top: 136, right: 256, bottom: 152 },
        Rect { left: 240, top: 136, right: 256, bottom: 152 },
        Rect { left: 256, top: 136, right: 272, bottom: 152 },
    ])
}

fn default_n102_malco_computer_wave() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 208, top: 120, right: 224, bottom: 136 },
        Rect { left: 224, top: 120, right: 240, bottom: 136 },
        Rect { left: 240, top: 120, right: 256, bottom: 136 },
        Rect { left: 256, top: 120, right: 272, bottom: 136 },
    ])
}

fn default_n103_mannan_projectile() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 192, top: 96, right: 208, bottom: 120 },
        Rect { left: 208, top: 96, right: 224, bottom: 120 },
        Rect { left: 224, top: 96, right: 240, bottom: 120 },
        Rect { left: 192, top: 120, right: 208, bottom: 144 },
        Rect { left: 208, top: 120, right: 224, bottom: 144 },
        Rect { left: 224, top: 120, right: 240, bottom: 144 },
    ])
}

fn default_n104_frog() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 112, right: 32, bottom: 144 },
        Rect { left: 32, top: 112, right: 64, bottom: 144 },
        Rect { left: 64, top: 112, right: 96, bottom: 144 },
        Rect { left: 0, top: 144, right: 32, bottom: 176 },
        Rect { left: 32, top: 144, right: 64, bottom: 176 },
        Rect { left: 64, top: 144, right: 96, bottom: 176 },
    ])
}

fn default_n105_hey_bubble_low() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 128, top: 32, right: 144, bottom: 48 },
        Rect { left: 128, top: 32, right: 128, bottom: 32 },
    ])
}

fn default_n107_malco_broken() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 144, top: 0, right: 160, bottom: 24 },
        Rect { left: 160, top: 0, right: 176, bottom: 24 },
        Rect { left: 176, top: 0, right: 192, bottom: 24 },
        Rect { left: 192, top: 0, right: 208, bottom: 24 },
        Rect { left: 208, top: 0, right: 224, bottom: 24 },
        Rect { left: 224, top: 0, right: 240, bottom: 24 },
        Rect { left: 176, top: 0, right: 192, bottom: 24 },
        Rect { left: 192, top: 0, right: 208, bottom: 24 },
        Rect { left: 208, top: 0, right: 224, bottom: 24 },
        Rect { left: 192, top: 0, right: 208, bottom: 24 },
    ])
}

fn default_n108_balfrog_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 96, top: 48, right: 112, bottom: 64 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
    ])
}

fn default_n109_malco_powered_on() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 240, top: 0, right: 256, bottom: 24 },
        Rect { left: 256, top: 0, right: 272, bottom: 24 },
        Rect { left: 240, top: 24, right: 256, bottom: 48 },
        Rect { left: 256, top: 24, right: 272, bottom: 48 },
    ])
}

fn default_n110_puchi() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 96, top: 128, right: 112, bottom: 144 },
        Rect { left: 112, top: 128, right: 128, bottom: 144 },
        Rect { left: 128, top: 128, right: 144, bottom: 144 },
        Rect { left: 96, top: 144, right: 112, bottom: 160 },
        Rect { left: 112, top: 144, right: 128, bottom: 160 },
        Rect { left: 128, top: 144, right: 144, bottom: 160 },
    ])
}

fn default_n111_quote_teleport_out() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
    ])
}

fn default_n112_quote_teleport_in() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
    ])
}

fn default_n113_professor_booster() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
        Rect { left: 240, top: 0, right: 256, bottom: 16 },
        Rect { left: 256, top: 0, right: 272, bottom: 16 },
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
        Rect { left: 272, top: 0, right: 288, bottom: 16 },
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
        Rect { left: 288, top: 0, right: 304, bottom: 16 },
        Rect { left: 224, top: 16, right: 240, bottom: 32 },
        Rect { left: 240, top: 16, right: 256, bottom: 32 },
        Rect { left: 256, top: 16, right: 272, bottom: 32 },
        Rect { left: 224, top: 16, right: 240, bottom: 32 },
        Rect { left: 272, top: 16, right: 288, bottom: 32 },
        Rect { left: 224, top: 16, right: 240, bottom: 32 },
        Rect { left: 288, top: 16, right: 304, bottom: 32 },
    ])
}

fn default_n114_press() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 144, top: 112, right: 160, bottom: 136 },
        Rect { left: 160, top: 112, right: 176, bottom: 136 },
        Rect { left: 176, top: 112, right: 192, bottom: 136 },
    ])
}

fn default_n115_ravil() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 120, right: 24, bottom: 144 },
        Rect { left: 24, top: 120, right: 48, bottom: 144 },
        Rect { left: 48, top: 120, right: 72, bottom: 144 },
        Rect { left: 72, top: 120, right: 96, bottom: 144 },
        Rect { left: 96, top: 120, right: 120, bottom: 144 },
        Rect { left: 120, top: 120, right: 144, bottom: 144 },
        Rect { left: 0, top: 144, right: 24, bottom: 168 },
        Rect { left: 24, top: 144, right: 48, bottom: 168 },
        Rect { left: 48, top: 144, right: 72, bottom: 168 },
        Rect { left: 72, top: 144, right: 96, bottom: 168 },
        Rect { left: 96, top: 144, right: 120, bottom: 168 },
        Rect { left: 120, top: 144, right: 144, bottom: 168 },
    ])
}

fn default_n116_red_petals() -> Rect<u16> {
    Rect { left: 272, top: 184, right: 320, bottom: 200 }
}

fn default_n117_curly() -> SafeNPCRect<20> {
    SafeNPCRect([
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 16, top: 96, right: 32, bottom: 112 },
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 176, top: 96, right: 192, bottom: 112 },
        Rect { left: 112, top: 96, right: 128, bottom: 112 },
        Rect { left: 160, top: 96, right: 176, bottom: 112 },
        Rect { left: 144, top: 96, right: 160, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 16, top: 112, right: 32, bottom: 128 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 32, top: 112, right: 48, bottom: 128 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 176, top: 112, right: 192, bottom: 128 },
        Rect { left: 112, top: 112, right: 128, bottom: 128 },
        Rect { left: 160, top: 112, right: 176, bottom: 128 },
        Rect { left: 144, top: 112, right: 160, bottom: 128 },
        Rect { left: 48, top: 112, right: 64, bottom: 128 },
    ])
}

fn default_n118_curly_boss() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 0, top: 32, right: 32, bottom: 56 },
        Rect { left: 32, top: 32, right: 64, bottom: 56 },
        Rect { left: 64, top: 32, right: 96, bottom: 56 },
        Rect { left: 96, top: 32, right: 128, bottom: 56 },
        Rect { left: 0, top: 32, right: 32, bottom: 56 },
        Rect { left: 128, top: 32, right: 160, bottom: 56 },
        Rect { left: 0, top: 32, right: 32, bottom: 56 },
        Rect { left: 0, top: 32, right: 32, bottom: 56 },
        Rect { left: 160, top: 32, right: 192, bottom: 56 },
        Rect { left: 0, top: 56, right: 32, bottom: 80 },
        Rect { left: 32, top: 56, right: 64, bottom: 80 },
        Rect { left: 64, top: 56, right: 96, bottom: 80 },
        Rect { left: 96, top: 56, right: 128, bottom: 80 },
        Rect { left: 0, top: 56, right: 32, bottom: 80 },
        Rect { left: 128, top: 56, right: 160, bottom: 80 },
        Rect { left: 0, top: 56, right: 32, bottom: 80 },
        Rect { left: 0, top: 56, right: 32, bottom: 80 },
        Rect { left: 160, top: 56, right: 192, bottom: 80 },
    ])
}

fn default_n119_table_chair() -> Rect<u16> {
    Rect { left: 248, top: 184, right: 272, bottom: 200 }
}

fn default_n120_colon_a() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 64, top: 0, right: 80, bottom: 16 }, Rect { left: 64, top: 16, right: 80, bottom: 32 }])
}

fn default_n121_colon_b() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
    ])
}

fn default_n122_colon_enraged() -> SafeNPCRect<20> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
    ])
}

fn default_n123_curly_boss_bullet() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 192, top: 0, right: 208, bottom: 16 },
        Rect { left: 208, top: 0, right: 224, bottom: 16 },
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
        Rect { left: 240, top: 0, right: 256, bottom: 16 },
    ])
}

fn default_n124_sunstone() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 160, top: 0, right: 192, bottom: 32 },
        Rect { left: 192, top: 0, right: 224, bottom: 32 },
    ])
}

fn default_n125_hidden_item() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 0, top: 96, right: 16, bottom: 112 }, Rect { left: 16, top: 96, right: 32, bottom: 112 }])
}

fn default_n126_puppy_running() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 48, top: 144, right: 64, bottom: 160 },
        Rect { left: 64, top: 144, right: 80, bottom: 160 },
        Rect { left: 48, top: 144, right: 64, bottom: 160 },
        Rect { left: 80, top: 144, right: 96, bottom: 160 },
        Rect { left: 96, top: 144, right: 112, bottom: 160 },
        Rect { left: 112, top: 144, right: 128, bottom: 160 },
        Rect { left: 48, top: 160, right: 64, bottom: 176 },
        Rect { left: 64, top: 160, right: 80, bottom: 176 },
        Rect { left: 48, top: 160, right: 64, bottom: 176 },
        Rect { left: 80, top: 160, right: 96, bottom: 176 },
        Rect { left: 96, top: 160, right: 112, bottom: 176 },
        Rect { left: 112, top: 160, right: 128, bottom: 176 },
    ])
}

fn default_n127_machine_gun_trail_l2() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 80, top: 80, right: 96, bottom: 96 },
        Rect { left: 96, top: 80, right: 112, bottom: 96 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 112, top: 64, right: 128, bottom: 80 },
        Rect { left: 112, top: 80, right: 128, bottom: 96 },
    ])
}

fn default_n128_machine_gun_trail_l3() -> SafeNPCRect<20> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 176, top: 16, right: 184, bottom: 32 },
        Rect { left: 184, top: 16, right: 192, bottom: 32 },
        Rect { left: 192, top: 16, right: 200, bottom: 32 },
        Rect { left: 200, top: 16, right: 208, bottom: 32 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 176, top: 32, right: 192, bottom: 40 },
        Rect { left: 176, top: 40, right: 192, bottom: 48 },
        Rect { left: 192, top: 32, right: 208, bottom: 40 },
        Rect { left: 192, top: 40, right: 208, bottom: 48 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 232, top: 16, right: 240, bottom: 32 },
        Rect { left: 224, top: 16, right: 232, bottom: 32 },
        Rect { left: 216, top: 16, right: 224, bottom: 32 },
        Rect { left: 208, top: 16, right: 216, bottom: 32 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 208, top: 32, right: 224, bottom: 40 },
        Rect { left: 208, top: 40, right: 224, bottom: 48 },
        Rect { left: 224, top: 32, right: 232, bottom: 40 },
        Rect { left: 224, top: 40, right: 232, bottom: 48 },
    ])
}

fn default_n129_fireball_snake_trail() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
        Rect { left: 144, top: 48, right: 160, bottom: 64 },
        Rect { left: 160, top: 48, right: 176, bottom: 64 },
        Rect { left: 128, top: 64, right: 144, bottom: 80 },
        Rect { left: 144, top: 64, right: 160, bottom: 80 },
        Rect { left: 160, top: 64, right: 176, bottom: 80 },
        Rect { left: 128, top: 80, right: 144, bottom: 96 },
        Rect { left: 144, top: 80, right: 160, bottom: 96 },
        Rect { left: 160, top: 80, right: 176, bottom: 96 },
        Rect { left: 176, top: 48, right: 192, bottom: 64 },
        Rect { left: 192, top: 48, right: 208, bottom: 64 },
        Rect { left: 208, top: 48, right: 224, bottom: 64 },
        Rect { left: 176, top: 64, right: 192, bottom: 80 },
        Rect { left: 192, top: 64, right: 208, bottom: 80 },
        Rect { left: 208, top: 64, right: 224, bottom: 80 },
        Rect { left: 176, top: 80, right: 192, bottom: 96 },
        Rect { left: 192, top: 80, right: 208, bottom: 96 },
        Rect { left: 208, top: 80, right: 224, bottom: 96 },
    ])
}

fn default_n130_puppy_sitting() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 48, top: 144, right: 64, bottom: 160 },
        Rect { left: 64, top: 144, right: 80, bottom: 160 },
        Rect { left: 48, top: 144, right: 64, bottom: 160 },
        Rect { left: 80, top: 144, right: 96, bottom: 160 },
        Rect { left: 48, top: 160, right: 64, bottom: 176 },
        Rect { left: 64, top: 160, right: 80, bottom: 176 },
        Rect { left: 48, top: 160, right: 64, bottom: 176 },
        Rect { left: 80, top: 160, right: 96, bottom: 176 },
    ])
}

fn default_n131_puppy_sleeping() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 144, top: 144, right: 160, bottom: 160 },
        Rect { left: 144, top: 160, right: 160, bottom: 176 },
    ])
}

fn default_n132_puppy_barking() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 48, top: 144, right: 64, bottom: 160 },
        Rect { left: 64, top: 144, right: 80, bottom: 160 },
        Rect { left: 96, top: 144, right: 112, bottom: 160 },
        Rect { left: 96, top: 144, right: 112, bottom: 160 },
        Rect { left: 128, top: 144, right: 144, bottom: 160 },
        Rect { left: 48, top: 160, right: 64, bottom: 176 },
        Rect { left: 64, top: 160, right: 80, bottom: 176 },
        Rect { left: 96, top: 160, right: 112, bottom: 176 },
        Rect { left: 96, top: 160, right: 112, bottom: 176 },
        Rect { left: 128, top: 160, right: 144, bottom: 176 },
    ])
}

fn default_n133_jenka() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 176, top: 32, right: 192, bottom: 48 },
        Rect { left: 192, top: 32, right: 208, bottom: 48 },
        Rect { left: 176, top: 48, right: 192, bottom: 64 },
        Rect { left: 192, top: 48, right: 208, bottom: 64 },
    ])
}

fn default_n134_armadillo() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 224, top: 0, right: 256, bottom: 16 },
        Rect { left: 256, top: 0, right: 288, bottom: 16 },
        Rect { left: 288, top: 0, right: 320, bottom: 16 },
        Rect { left: 224, top: 16, right: 256, bottom: 32 },
        Rect { left: 256, top: 16, right: 288, bottom: 32 },
        Rect { left: 288, top: 16, right: 320, bottom: 32 },
    ])
}

fn default_n135_skeleton() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 256, top: 32, right: 288, bottom: 64 },
        Rect { left: 288, top: 32, right: 320, bottom: 64 },
        Rect { left: 256, top: 64, right: 288, bottom: 96 },
        Rect { left: 288, top: 64, right: 320, bottom: 96 },
    ])
}

fn default_n136_puppy_carried() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 48, top: 144, right: 64, bottom: 160 },
        Rect { left: 64, top: 144, right: 80, bottom: 160 },
        Rect { left: 48, top: 160, right: 64, bottom: 176 },
        Rect { left: 64, top: 160, right: 80, bottom: 176 },
    ])
}

fn default_n137_large_door_frame() -> Rect<u16> {
    Rect { left: 96, top: 136, right: 128, bottom: 188 }
}

fn default_n138_large_door() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 96, top: 112, right: 112, bottom: 136 },
        Rect { left: 112, top: 112, right: 128, bottom: 136 },
    ])
}

fn default_n139_doctor() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 128, right: 24, bottom: 160 },
        Rect { left: 24, top: 128, right: 48, bottom: 160 },
        Rect { left: 48, top: 128, right: 72, bottom: 160 },
        Rect { left: 0, top: 160, right: 24, bottom: 192 },
        Rect { left: 24, top: 160, right: 48, bottom: 192 },
        Rect { left: 48, top: 160, right: 72, bottom: 192 },
    ])
}

fn default_n140_toroko_frenzied() -> SafeNPCRect<28> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 32, bottom: 32 },
        Rect { left: 32, top: 0, right: 64, bottom: 32 },
        Rect { left: 64, top: 0, right: 96, bottom: 32 },
        Rect { left: 96, top: 0, right: 128, bottom: 32 },
        Rect { left: 128, top: 0, right: 160, bottom: 32 },
        Rect { left: 160, top: 0, right: 192, bottom: 32 },
        Rect { left: 192, top: 0, right: 224, bottom: 32 },
        Rect { left: 224, top: 0, right: 256, bottom: 32 },
        Rect { left: 0, top: 64, right: 32, bottom: 96 },
        Rect { left: 32, top: 64, right: 64, bottom: 96 },
        Rect { left: 64, top: 64, right: 96, bottom: 96 },
        Rect { left: 96, top: 64, right: 128, bottom: 96 },
        Rect { left: 128, top: 64, right: 160, bottom: 96 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 0, top: 32, right: 32, bottom: 64 },
        Rect { left: 32, top: 32, right: 64, bottom: 64 },
        Rect { left: 64, top: 32, right: 96, bottom: 64 },
        Rect { left: 96, top: 32, right: 128, bottom: 64 },
        Rect { left: 128, top: 32, right: 160, bottom: 64 },
        Rect { left: 160, top: 32, right: 192, bottom: 64 },
        Rect { left: 192, top: 32, right: 224, bottom: 64 },
        Rect { left: 224, top: 32, right: 256, bottom: 64 },
        Rect { left: 0, top: 96, right: 32, bottom: 128 },
        Rect { left: 32, top: 96, right: 64, bottom: 128 },
        Rect { left: 64, top: 96, right: 96, bottom: 128 },
        Rect { left: 96, top: 96, right: 128, bottom: 128 },
        Rect { left: 128, top: 96, right: 160, bottom: 128 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
    ])
}

fn default_n141_toroko_block_projectile() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 288, top: 32, right: 304, bottom: 48 },
        Rect { left: 304, top: 32, right: 320, bottom: 48 },
    ])
}

fn default_n142_flower_cub() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 0, top: 128, right: 16, bottom: 144 },
        Rect { left: 16, top: 128, right: 32, bottom: 144 },
        Rect { left: 32, top: 128, right: 48, bottom: 144 },
        Rect { left: 48, top: 128, right: 64, bottom: 144 },
        Rect { left: 64, top: 128, right: 80, bottom: 144 },
    ])
}

fn default_n143_jenka_collapsed() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 208, top: 32, right: 224, bottom: 48 },
        Rect { left: 208, top: 48, right: 224, bottom: 64 },
    ])
}

fn default_n144_toroko_teleporting_in() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 32, top: 64, right: 48, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 128, top: 64, right: 144, bottom: 80 },
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 128, top: 80, right: 144, bottom: 96 },
    ])
}

fn default_n145_king_sword() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 96, top: 32, right: 112, bottom: 48 },
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
    ])
}

fn default_n146_lightning() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 256, top: 0, right: 272, bottom: 240 },
        Rect { left: 272, top: 0, right: 288, bottom: 240 },
        Rect { left: 288, top: 0, right: 304, bottom: 240 },
        Rect { left: 304, top: 0, right: 320, bottom: 240 },
    ])
}

fn default_n147_critter_purple() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 16, top: 96, right: 32, bottom: 112 },
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 64, top: 96, right: 80, bottom: 112 },
        Rect { left: 80, top: 96, right: 96, bottom: 112 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 16, top: 112, right: 32, bottom: 128 },
        Rect { left: 32, top: 112, right: 48, bottom: 128 },
        Rect { left: 48, top: 112, right: 64, bottom: 128 },
        Rect { left: 64, top: 112, right: 80, bottom: 128 },
        Rect { left: 80, top: 112, right: 96, bottom: 128 },
    ])
}

fn default_n148_critter_purple_projectile() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 96, top: 96, right: 104, bottom: 104 },
        Rect { left: 104, top: 96, right: 112, bottom: 104 },
    ])
}

fn default_n149_horizontal_moving_block() -> Rect<u16> {
    Rect { left: 16, top: 0, right: 48, bottom: 32 }
}

fn default_n150_quote() -> SafeNPCRect<20> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
    ])
}

fn default_n151_blue_robot_standing() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 192, top: 0, right: 208, bottom: 16 },
        Rect { left: 208, top: 0, right: 224, bottom: 16 },
        Rect { left: 192, top: 16, right: 208, bottom: 32 },
        Rect { left: 208, top: 16, right: 224, bottom: 32 },
    ])
}

fn default_n153_gaudi() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 24, bottom: 24 },    // 0 0 // left
        Rect { left: 24, top: 0, right: 48, bottom: 24 },   // 1 1
        Rect { left: 48, top: 0, right: 72, bottom: 24 },   // 2 2
        Rect { left: 0, top: 0, right: 24, bottom: 24 },    // 3 3
        Rect { left: 72, top: 0, right: 96, bottom: 24 },   // 4 4
        Rect { left: 0, top: 0, right: 24, bottom: 24 },    // 5 5
        Rect { left: 96, top: 48, right: 120, bottom: 72 }, // 6 20
        Rect { left: 0, top: 24, right: 24, bottom: 48 },   // 0 0 // right
        Rect { left: 24, top: 24, right: 48, bottom: 48 },  // 1 1
        Rect { left: 48, top: 24, right: 72, bottom: 48 },  // 2 2
        Rect { left: 0, top: 24, right: 24, bottom: 48 },   // 3 3
        Rect { left: 72, top: 24, right: 96, bottom: 48 },  // 4 4
        Rect { left: 0, top: 24, right: 24, bottom: 48 },   // 5 5
        Rect { left: 96, top: 72, right: 120, bottom: 96 }, // 6 20
    ])
}

fn default_n154_gaudi_dead() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 168, top: 24, right: 192, bottom: 48 },
        Rect { left: 192, top: 24, right: 216, bottom: 48 },
        Rect { left: 216, top: 24, right: 240, bottom: 48 },
        Rect { left: 168, top: 0, right: 192, bottom: 24 },
        Rect { left: 192, top: 0, right: 216, bottom: 24 },
        Rect { left: 216, top: 0, right: 240, bottom: 24 },
    ])
}

fn default_n155_gaudi_flying() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 48, right: 24, bottom: 72 },    // 0 14 // left
        Rect { left: 24, top: 48, right: 48, bottom: 72 },   // 1 15
        Rect { left: 288, top: 0, right: 312, bottom: 24 },  // 2 18
        Rect { left: 24, top: 48, right: 48, bottom: 72 },   // 3 19
        Rect { left: 0, top: 72, right: 24, bottom: 96 },    // 0 14 // right
        Rect { left: 24, top: 72, right: 48, bottom: 96 },   // 1 15
        Rect { left: 288, top: 24, right: 312, bottom: 48 }, // 2 18
        Rect { left: 24, top: 72, right: 48, bottom: 96 },   // 3 19
    ])
}

fn default_n156_gaudi_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 96, top: 112, right: 112, bottom: 128 },
        Rect { left: 112, top: 112, right: 128, bottom: 128 },
        Rect { left: 128, top: 112, right: 144, bottom: 128 },
    ])
}

fn default_n157_vertical_moving_block() -> Rect<u16> {
    Rect { left: 16, top: 0, right: 48, bottom: 32 }
}

fn default_n158_fish_missile() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 224, right: 16, bottom: 240 },
        Rect { left: 16, top: 224, right: 32, bottom: 240 },
        Rect { left: 32, top: 224, right: 48, bottom: 240 },
        Rect { left: 48, top: 224, right: 64, bottom: 240 },
        Rect { left: 64, top: 224, right: 80, bottom: 240 },
        Rect { left: 80, top: 224, right: 96, bottom: 240 },
        Rect { left: 96, top: 224, right: 112, bottom: 240 },
        Rect { left: 112, top: 224, right: 128, bottom: 240 },
    ])
}

fn default_n159_monster_x_defeated() -> Rect<u16> {
    Rect { left: 144, top: 128, right: 192, bottom: 200 }
}

fn default_n160_puu_black() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 40, top: 0, right: 80, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 40, top: 24, right: 80, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
    ])
}

fn default_n161_puu_black_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 16, top: 48, right: 32, bottom: 64 },
        Rect { left: 32, top: 48, right: 48, bottom: 64 },
    ])
}

fn default_n162_puu_black_dead() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 40, top: 0, right: 80, bottom: 24 },
        Rect { left: 40, top: 24, right: 80, bottom: 48 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
    ])
}

fn default_n163_dr_gero() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 192, top: 0, right: 208, bottom: 16 },
        Rect { left: 208, top: 0, right: 224, bottom: 16 },
        Rect { left: 192, top: 16, right: 208, bottom: 32 },
        Rect { left: 208, top: 16, right: 224, bottom: 32 },
    ])
}

fn default_n164_nurse_hasumi() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
        Rect { left: 240, top: 0, right: 256, bottom: 16 },
        Rect { left: 224, top: 16, right: 240, bottom: 32 },
        Rect { left: 240, top: 16, right: 256, bottom: 32 },
    ])
}

fn default_n165_curly_collapsed() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 144, top: 96, right: 160, bottom: 112 },
        Rect { left: 192, top: 96, right: 208, bottom: 112 },
        Rect { left: 208, top: 96, right: 224, bottom: 112 },
    ])
}

fn default_n166_chaba() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 144, top: 104, right: 184, bottom: 128 },
        Rect { left: 184, top: 104, right: 224, bottom: 128 },
    ])
}

fn default_n167_booster_falling() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 304, top: 0, right: 320, bottom: 16 },
        Rect { left: 304, top: 16, right: 320, bottom: 32 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
    ])
}

fn default_n168_boulder() -> Rect<u16> {
    Rect { left: 264, top: 56, right: 320, bottom: 96 }
}

fn default_n169_balrog_shooting_missiles() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 0, top: 48, right: 40, bottom: 72 },
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 40, top: 48, right: 80, bottom: 72 },
        Rect { left: 0, top: 0, right: 40, bottom: 24 },
        Rect { left: 80, top: 48, right: 120, bottom: 72 },
        Rect { left: 120, top: 48, right: 160, bottom: 72 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 0, top: 72, right: 40, bottom: 96 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 40, top: 72, right: 80, bottom: 96 },
        Rect { left: 0, top: 24, right: 40, bottom: 48 },
        Rect { left: 80, top: 72, right: 120, bottom: 96 },
        Rect { left: 120, top: 72, right: 160, bottom: 96 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
    ])
}

fn default_n170_balrog_missile() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 112, top: 96, right: 128, bottom: 104 },
        Rect { left: 128, top: 96, right: 144, bottom: 104 },
        Rect { left: 112, top: 104, right: 128, bottom: 112 },
        Rect { left: 128, top: 104, right: 144, bottom: 112 },
    ])
}

fn default_n171_fire_whirrr() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 120, top: 48, right: 152, bottom: 80 },
        Rect { left: 152, top: 48, right: 184, bottom: 80 },
        Rect { left: 184, top: 48, right: 216, bottom: 80 },
        Rect { left: 216, top: 48, right: 248, bottom: 80 },
    ])
}

fn default_n172_fire_whirrr_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 248, top: 48, right: 264, bottom: 80 },
        Rect { left: 264, top: 48, right: 280, bottom: 80 },
        Rect { left: 280, top: 48, right: 296, bottom: 80 },
    ])
}

fn default_n173_gaudi_armored() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 128, right: 24, bottom: 152 },
        Rect { left: 24, top: 128, right: 48, bottom: 152 },
        Rect { left: 48, top: 128, right: 72, bottom: 152 },
        Rect { left: 72, top: 128, right: 96, bottom: 152 },
        Rect { left: 0, top: 152, right: 24, bottom: 176 },
        Rect { left: 24, top: 152, right: 48, bottom: 176 },
        Rect { left: 48, top: 152, right: 72, bottom: 176 },
        Rect { left: 72, top: 152, right: 96, bottom: 176 },
    ])
}

fn default_n174_gaudi_armored_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 120, top: 80, right: 136, bottom: 96 },
        Rect { left: 136, top: 80, right: 152, bottom: 96 },
        Rect { left: 152, top: 80, right: 168, bottom: 96 },
    ])
}

fn default_n175_gaudi_egg() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 168, top: 80, right: 192, bottom: 104 },
        Rect { left: 192, top: 80, right: 216, bottom: 104 },
        Rect { left: 216, top: 80, right: 240, bottom: 104 },
        Rect { left: 240, top: 80, right: 264, bottom: 104 },
    ])
}

fn default_n176_buyo_buyo_base() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 96, top: 128, right: 128, bottom: 144 },
        Rect { left: 128, top: 128, right: 160, bottom: 144 },
        Rect { left: 160, top: 128, right: 192, bottom: 144 },
        Rect { left: 96, top: 144, right: 128, bottom: 160 },
        Rect { left: 128, top: 144, right: 160, bottom: 160 },
        Rect { left: 160, top: 144, right: 192, bottom: 160 },
    ])
}

fn default_n177_buyo_buyo() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 192, top: 128, right: 208, bottom: 144 },
        Rect { left: 208, top: 128, right: 224, bottom: 144 },
    ])
}

fn default_n178_core_blade_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 224, right: 16, bottom: 240 },
        Rect { left: 16, top: 224, right: 32, bottom: 240 },
        Rect { left: 32, top: 224, right: 48, bottom: 240 },
    ])
}

fn default_n179_core_wisp_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 48, top: 224, right: 72, bottom: 240 },
        Rect { left: 72, top: 224, right: 96, bottom: 240 },
        Rect { left: 96, top: 224, right: 120, bottom: 240 },
    ])
}

fn default_n180_curly_ai() -> SafeNPCRect<22> {
    SafeNPCRect([
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 16, top: 96, right: 32, bottom: 112 },
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 64, top: 96, right: 80, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 80, top: 96, right: 96, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 144, top: 96, right: 160, bottom: 112 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 16, top: 112, right: 32, bottom: 128 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 32, top: 112, right: 48, bottom: 128 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 48, top: 112, right: 64, bottom: 128 },
        Rect { left: 64, top: 112, right: 80, bottom: 128 },
        Rect { left: 48, top: 112, right: 64, bottom: 128 },
        Rect { left: 80, top: 112, right: 96, bottom: 128 },
        Rect { left: 48, top: 112, right: 64, bottom: 128 },
        Rect { left: 144, top: 112, right: 160, bottom: 128 },
    ])
}

fn default_n181_curly_ai_machine_gun() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 216, top: 152, right: 232, bottom: 168 },
        Rect { left: 232, top: 152, right: 248, bottom: 168 },
        Rect { left: 216, top: 168, right: 232, bottom: 184 },
        Rect { left: 232, top: 168, right: 248, bottom: 184 },
    ])
}

fn default_n182_curly_ai_polar_star() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 184, top: 152, right: 200, bottom: 168 },
        Rect { left: 200, top: 152, right: 216, bottom: 168 },
        Rect { left: 184, top: 168, right: 200, bottom: 184 },
        Rect { left: 200, top: 168, right: 216, bottom: 184 },
    ])
}

fn default_n183_curly_air_tank_bubble() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 56, top: 96, right: 80, bottom: 120 },
        Rect { left: 80, top: 96, right: 104, bottom: 120 },
    ])
}

fn default_n184_shutter() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 32, bottom: 96 },
        Rect { left: 32, top: 64, right: 64, bottom: 96 },
        Rect { left: 64, top: 64, right: 96, bottom: 96 },
        Rect { left: 32, top: 64, right: 64, bottom: 96 },
    ])
}

fn default_n185_small_shutter() -> Rect<u16> {
    Rect { left: 96, top: 64, right: 112, bottom: 96 }
}

fn default_n186_lift_block() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 48, top: 48, right: 64, bottom: 64 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
        Rect { left: 80, top: 48, right: 96, bottom: 64 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
    ])
}

fn default_n187_fuzz_core() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 224, top: 104, right: 256, bottom: 136 },
        Rect { left: 256, top: 104, right: 288, bottom: 136 },
        Rect { left: 224, top: 136, right: 256, bottom: 168 },
        Rect { left: 256, top: 136, right: 288, bottom: 168 },
    ])
}

fn default_n188_fuzz() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 288, top: 104, right: 304, bottom: 120 },
        Rect { left: 304, top: 104, right: 320, bottom: 120 },
        Rect { left: 288, top: 120, right: 304, bottom: 136 },
        Rect { left: 304, top: 120, right: 320, bottom: 136 },
    ])
}

fn default_n189_homing_flame() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 224, top: 184, right: 232, bottom: 200 },
        Rect { left: 232, top: 184, right: 240, bottom: 200 },
        Rect { left: 240, top: 184, right: 248, bottom: 200 },
    ])
}

fn default_n190_broken_robot() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 192, top: 32, right: 208, bottom: 48 },
        Rect { left: 208, top: 32, right: 224, bottom: 48 },
    ])
}

fn default_n192_scooter() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 224, top: 64, right: 256, bottom: 80 },
        Rect { left: 256, top: 64, right: 288, bottom: 96 },
        Rect { left: 224, top: 80, right: 256, bottom: 96 },
        Rect { left: 288, top: 64, right: 320, bottom: 96 },
    ])
}

fn default_n193_broken_scooter() -> Rect<u16> {
    Rect { left: 256, top: 96, right: 320, bottom: 112 }
}

fn default_n194_broken_blue_robot() -> Rect<u16> {
    Rect { left: 192, top: 120, right: 224, bottom: 128 }
}

fn default_n195_background_grate() -> Rect<u16> {
    Rect { left: 112, top: 64, right: 128, bottom: 80 }
}

fn default_n196_ironhead_wall() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 112, top: 64, right: 144, bottom: 80 },
        Rect { left: 112, top: 80, right: 144, bottom: 96 },
    ])
}

fn default_n197_porcupine_fish() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
    ])
}

fn default_n198_ironhead_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 208, top: 48, right: 224, bottom: 72 },
        Rect { left: 224, top: 48, right: 240, bottom: 72 },
        Rect { left: 240, top: 48, right: 256, bottom: 72 },
    ])
}

fn default_n199_wind_particles() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 72, top: 16, right: 74, bottom: 18 },
        Rect { left: 74, top: 16, right: 76, bottom: 18 },
        Rect { left: 76, top: 16, right: 78, bottom: 18 },
        Rect { left: 78, top: 16, right: 80, bottom: 18 },
        Rect { left: 80, top: 16, right: 82, bottom: 18 },
    ])
}

fn default_n200_zombie_dragon() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 40, top: 0, right: 80, bottom: 40 },
        Rect { left: 80, top: 0, right: 120, bottom: 40 },
        Rect { left: 120, top: 0, right: 160, bottom: 40 },
        Rect { left: 160, top: 0, right: 200, bottom: 40 },
        Rect { left: 200, top: 0, right: 240, bottom: 40 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 40, top: 40, right: 80, bottom: 80 },
        Rect { left: 80, top: 40, right: 120, bottom: 80 },
        Rect { left: 120, top: 40, right: 160, bottom: 80 },
        Rect { left: 160, top: 40, right: 200, bottom: 80 },
        Rect { left: 200, top: 40, right: 240, bottom: 80 },
    ])
}

fn default_n201_zombie_dragon_dead() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 200, top: 0, right: 240, bottom: 40 },
        Rect { left: 200, top: 40, right: 240, bottom: 80 },
    ])
}

fn default_n202_zombie_dragon_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 184, top: 216, right: 200, bottom: 240 },
        Rect { left: 200, top: 216, right: 216, bottom: 240 },
        Rect { left: 216, top: 216, right: 232, bottom: 240 },
    ])
}

fn default_n203_critter_destroyed_egg_corridor() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 0, top: 96, right: 16, bottom: 112 },
        Rect { left: 16, top: 96, right: 32, bottom: 112 },
        Rect { left: 32, top: 96, right: 48, bottom: 112 },
    ])
}

fn default_n204_small_falling_spike() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 240, top: 80, right: 256, bottom: 96 },
        Rect { left: 240, top: 144, right: 256, bottom: 160 },
    ])
}

fn default_n205_large_falling_spike() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 112, top: 80, right: 128, bottom: 112 },
        Rect { left: 128, top: 80, right: 144, bottom: 112 },
    ])
}

fn default_n206_counter_bomb() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 80, top: 80, right: 120, bottom: 120 },
        Rect { left: 120, top: 80, right: 160, bottom: 120 },
        Rect { left: 160, top: 80, right: 200, bottom: 120 },
    ])
}

fn default_n207_counter_bomb_countdown() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 0, top: 144, right: 16, bottom: 160 },
        Rect { left: 16, top: 144, right: 32, bottom: 160 },
        Rect { left: 32, top: 144, right: 48, bottom: 160 },
        Rect { left: 48, top: 144, right: 64, bottom: 160 },
        Rect { left: 64, top: 144, right: 80, bottom: 160 },
    ])
}

fn default_n208_basu_destroyed_egg_corridor() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 248, top: 80, right: 272, bottom: 104 },
        Rect { left: 272, top: 80, right: 296, bottom: 104 },
        Rect { left: 296, top: 80, right: 320, bottom: 104 },
        Rect { left: 248, top: 104, right: 272, bottom: 128 },
        Rect { left: 272, top: 104, right: 296, bottom: 128 },
        Rect { left: 296, top: 104, right: 320, bottom: 128 },
    ])
}

fn default_n209_basu_projectile_destroyed_egg_corridor() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 232, top: 96, right: 248, bottom: 112 },
        Rect { left: 200, top: 112, right: 216, bottom: 128 },
        Rect { left: 216, top: 112, right: 232, bottom: 128 },
        Rect { left: 232, top: 112, right: 248, bottom: 128 },
    ])
}

fn default_n210_beetle_destroyed_egg_corridor() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 16, top: 112, right: 32, bottom: 128 },
        Rect { left: 32, top: 112, right: 48, bottom: 128 },
        Rect { left: 48, top: 112, right: 64, bottom: 128 },
    ])
}

fn default_n211_small_spikes() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 256, top: 200, right: 272, bottom: 216 },
        Rect { left: 272, top: 200, right: 288, bottom: 216 },
        Rect { left: 288, top: 200, right: 304, bottom: 216 },
        Rect { left: 304, top: 200, right: 320, bottom: 216 },
    ])
}

fn default_n212_sky_dragon() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 160, top: 152, right: 200, bottom: 192 },
        Rect { left: 200, top: 152, right: 240, bottom: 192 },
        Rect { left: 240, top: 112, right: 280, bottom: 152 },
        Rect { left: 280, top: 112, right: 320, bottom: 152 },
    ])
}

fn default_n213_night_spirit() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 0, top: 0, right: 48, bottom: 48 },
        Rect { left: 48, top: 0, right: 96, bottom: 48 },
        Rect { left: 96, top: 0, right: 144, bottom: 48 },
        Rect { left: 144, top: 0, right: 192, bottom: 48 },
        Rect { left: 192, top: 0, right: 240, bottom: 48 },
        Rect { left: 240, top: 0, right: 288, bottom: 48 },
        Rect { left: 0, top: 48, right: 48, bottom: 96 },
        Rect { left: 48, top: 48, right: 96, bottom: 96 },
        Rect { left: 96, top: 48, right: 144, bottom: 96 },
    ])
}

fn default_n214_night_spirit_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 144, top: 48, right: 176, bottom: 64 },
        Rect { left: 176, top: 48, right: 208, bottom: 64 },
        Rect { left: 208, top: 48, right: 240, bottom: 64 },
    ])
}

fn default_n215_sandcroc_outer_wall() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 0, top: 96, right: 48, bottom: 128 },
        Rect { left: 48, top: 96, right: 96, bottom: 128 },
        Rect { left: 96, top: 96, right: 144, bottom: 128 },
        Rect { left: 144, top: 96, right: 192, bottom: 128 },
    ])
}

fn default_n216_debug_cat() -> Rect<u16> {
    Rect { left: 256, top: 192, right: 272, bottom: 216 }
}

fn default_n217_itoh() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 144, top: 64, right: 160, bottom: 80 },
        Rect { left: 160, top: 64, right: 176, bottom: 80 },
        Rect { left: 176, top: 64, right: 192, bottom: 80 },
        Rect { left: 192, top: 64, right: 208, bottom: 80 },
        Rect { left: 144, top: 80, right: 160, bottom: 96 },
        Rect { left: 160, top: 80, right: 176, bottom: 96 },
        Rect { left: 144, top: 80, right: 160, bottom: 96 },
        Rect { left: 176, top: 80, right: 192, bottom: 96 },
    ])
}

fn default_n218_core_giant_ball() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 256, top: 120, right: 288, bottom: 152 },
        Rect { left: 288, top: 120, right: 320, bottom: 152 },
    ])
}

fn default_n220_shovel_brigade() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
    ])
}

fn default_n221_shovel_brigade_walking() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 16, top: 64, right: 32, bottom: 80 },
        Rect { left: 32, top: 64, right: 48, bottom: 80 },
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 48, top: 64, right: 64, bottom: 80 },
        Rect { left: 0, top: 64, right: 16, bottom: 80 },
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 16, top: 80, right: 32, bottom: 96 },
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 48, top: 80, right: 64, bottom: 96 },
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
    ])
}

fn default_n222_prison_bars() -> Rect<u16> {
    Rect { left: 96, top: 168, right: 112, bottom: 200 }
}

fn default_n223_momorin() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 80, top: 192, right: 96, bottom: 216 },
        Rect { left: 96, top: 192, right: 112, bottom: 216 },
        Rect { left: 112, top: 192, right: 128, bottom: 216 },
        Rect { left: 80, top: 216, right: 96, bottom: 240 },
        Rect { left: 96, top: 216, right: 112, bottom: 240 },
        Rect { left: 112, top: 216, right: 128, bottom: 240 },
    ])
}

fn default_n224_chie() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
        Rect { left: 128, top: 32, right: 144, bottom: 48 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
    ])
}

fn default_n225_megane() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 64, top: 64, right: 80, bottom: 80 },
        Rect { left: 80, top: 64, right: 96, bottom: 80 },
        Rect { left: 64, top: 80, right: 80, bottom: 96 },
        Rect { left: 80, top: 80, right: 96, bottom: 96 },
    ])
}

fn default_n226_kanpachi_plantation() -> SafeNPCRect<7> {
    SafeNPCRect([
        Rect { left: 256, top: 56, right: 272, bottom: 80 },
        Rect { left: 272, top: 56, right: 288, bottom: 80 },
        Rect { left: 288, top: 56, right: 304, bottom: 80 },
        Rect { left: 256, top: 56, right: 272, bottom: 80 },
        Rect { left: 304, top: 56, right: 320, bottom: 80 },
        Rect { left: 256, top: 56, right: 272, bottom: 80 },
        Rect { left: 240, top: 56, right: 256, bottom: 80 },
    ])
}

fn default_n227_bucket() -> Rect<u16> {
    Rect { left: 208, top: 32, right: 224, bottom: 48 }
}

fn default_n228_droll() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 32, bottom: 40 },
        Rect { left: 32, top: 0, right: 64, bottom: 40 },
        Rect { left: 64, top: 0, right: 96, bottom: 40 },
        Rect { left: 96, top: 0, right: 128, bottom: 40 },
        Rect { left: 0, top: 40, right: 32, bottom: 80 },
        Rect { left: 32, top: 40, right: 64, bottom: 80 },
        Rect { left: 64, top: 40, right: 96, bottom: 80 },
        Rect { left: 96, top: 40, right: 128, bottom: 80 },
    ])
}

fn default_n229_red_flowers_sprouts() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 0, top: 96, right: 48, bottom: 112 }, Rect { left: 0, top: 112, right: 48, bottom: 128 }])
}

fn default_n230_red_flowers_blooming() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 48, top: 96, right: 96, bottom: 128 },
        Rect { left: 96, top: 96, right: 144, bottom: 128 },
    ])
}

fn default_n231_rocket() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 176, top: 32, right: 208, bottom: 48 },
        Rect { left: 176, top: 48, right: 208, bottom: 64 },
    ])
}

fn default_n232_orangebell() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 160, bottom: 32 },
        Rect { left: 160, top: 0, right: 192, bottom: 32 },
        Rect { left: 192, top: 0, right: 224, bottom: 32 },
        Rect { left: 128, top: 32, right: 160, bottom: 64 },
        Rect { left: 160, top: 32, right: 192, bottom: 64 },
        Rect { left: 192, top: 32, right: 224, bottom: 64 },
    ])
}

fn default_n233_orangebell_bat() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 256, top: 0, right: 272, bottom: 16 },
        Rect { left: 272, top: 0, right: 288, bottom: 16 },
        Rect { left: 288, top: 0, right: 304, bottom: 16 },
        Rect { left: 304, top: 0, right: 320, bottom: 16 },
        Rect { left: 256, top: 16, right: 272, bottom: 32 },
        Rect { left: 272, top: 16, right: 288, bottom: 32 },
        Rect { left: 288, top: 16, right: 304, bottom: 32 },
        Rect { left: 304, top: 16, right: 320, bottom: 32 },
    ])
}

fn default_n234_red_flowers_picked() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 144, top: 96, right: 192, bottom: 112 },
        Rect { left: 144, top: 112, right: 192, bottom: 128 },
    ])
}

fn default_n235_midorin() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 192, top: 96, right: 208, bottom: 112 },
        Rect { left: 208, top: 96, right: 224, bottom: 112 },
        Rect { left: 224, top: 96, right: 240, bottom: 112 },
        Rect { left: 192, top: 96, right: 208, bottom: 112 },
        Rect { left: 192, top: 112, right: 208, bottom: 128 },
        Rect { left: 208, top: 112, right: 224, bottom: 128 },
        Rect { left: 224, top: 112, right: 240, bottom: 128 },
        Rect { left: 192, top: 112, right: 208, bottom: 128 },
    ])
}

fn default_n236_gunfish() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 128, top: 64, right: 152, bottom: 88 },
        Rect { left: 152, top: 64, right: 176, bottom: 88 },
        Rect { left: 176, top: 64, right: 200, bottom: 88 },
        Rect { left: 200, top: 64, right: 224, bottom: 88 },
        Rect { left: 224, top: 64, right: 248, bottom: 88 },
        Rect { left: 248, top: 64, right: 272, bottom: 88 },
        Rect { left: 128, top: 88, right: 152, bottom: 112 },
        Rect { left: 152, top: 88, right: 176, bottom: 112 },
        Rect { left: 176, top: 88, right: 200, bottom: 112 },
        Rect { left: 200, top: 88, right: 224, bottom: 112 },
        Rect { left: 224, top: 88, right: 248, bottom: 112 },
        Rect { left: 248, top: 88, right: 272, bottom: 112 },
    ])
}

fn default_n237_gunfish_projectile() -> Rect<u16> {
    Rect { left: 312, top: 32, right: 320, bottom: 40 }
}

fn default_n238_press_sideways() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 184, top: 200, right: 208, bottom: 216 },
        Rect { left: 208, top: 200, right: 232, bottom: 216 },
        Rect { left: 232, top: 200, right: 256, bottom: 216 },
    ])
}

fn default_n239_cage_bars() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 192, top: 48, right: 256, bottom: 80 },
        Rect { left: 96, top: 112, right: 144, bottom: 144 },
    ])
}

fn default_n240_mimiga_jailed() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 160, top: 64, right: 176, bottom: 80 },
        Rect { left: 176, top: 64, right: 192, bottom: 80 },
        Rect { left: 192, top: 64, right: 208, bottom: 80 },
        Rect { left: 160, top: 64, right: 176, bottom: 80 },
        Rect { left: 208, top: 64, right: 224, bottom: 80 },
        Rect { left: 160, top: 64, right: 176, bottom: 80 },
        Rect { left: 160, top: 80, right: 176, bottom: 96 },
        Rect { left: 176, top: 80, right: 192, bottom: 96 },
        Rect { left: 192, top: 80, right: 208, bottom: 96 },
        Rect { left: 160, top: 80, right: 176, bottom: 96 },
        Rect { left: 208, top: 80, right: 224, bottom: 96 },
        Rect { left: 160, top: 80, right: 176, bottom: 96 },
    ])
}

fn default_n241_critter_red() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
    ])
}

fn default_n242_bat_last_cave() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 32, top: 32, right: 48, bottom: 48 },
        Rect { left: 48, top: 32, right: 64, bottom: 48 },
        Rect { left: 64, top: 32, right: 80, bottom: 48 },
        Rect { left: 80, top: 32, right: 96, bottom: 48 },
        Rect { left: 32, top: 48, right: 48, bottom: 64 },
        Rect { left: 48, top: 48, right: 64, bottom: 64 },
        Rect { left: 64, top: 48, right: 80, bottom: 64 },
        Rect { left: 80, top: 48, right: 96, bottom: 64 },
    ])
}

fn default_n244_lava_drop() -> Rect<u16> {
    Rect { left: 96, top: 0, right: 104, bottom: 16 }
}

fn default_n245_lava_drop_generator() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 104, top: 0, right: 112, bottom: 16 },
        Rect { left: 112, top: 0, right: 120, bottom: 16 },
        Rect { left: 120, top: 0, right: 128, bottom: 16 },
    ])
}

fn default_n246_press_proximity() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 144, top: 112, right: 160, bottom: 136 },
        Rect { left: 160, top: 112, right: 176, bottom: 136 },
        Rect { left: 176, top: 112, right: 192, bottom: 136 },
    ])
}

fn default_n247_misery_boss() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 32, top: 0, right: 48, bottom: 16 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
        Rect { left: 64, top: 0, right: 80, bottom: 16 },
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 96, top: 0, right: 112, bottom: 16 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 64, top: 16, right: 80, bottom: 32 },
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
        Rect { left: 96, top: 16, right: 112, bottom: 32 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 112, top: 16, right: 128, bottom: 32 },
    ])
}

fn default_n248_misery_boss_vanishing() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 16, top: 48, right: 32, bottom: 64 },
        Rect { left: 32, top: 48, right: 48, bottom: 64 },
    ])
}

fn default_n249_misery_boss_appearing() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 48, top: 48, right: 64, bottom: 64 }, Rect { left: 64, top: 48, right: 80, bottom: 64 }])
}

fn default_n250_misery_boss_lightning_ball() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 32, right: 16, bottom: 48 },
        Rect { left: 16, top: 32, right: 32, bottom: 48 },
        Rect { left: 32, top: 32, right: 48, bottom: 48 },
    ])
}

fn default_n251_misery_boss_lightning() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 80, top: 32, right: 96, bottom: 64 }, Rect { left: 96, top: 32, right: 112, bottom: 64 }])
}

fn default_n252_misery_boss_bats() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 48, top: 32, right: 64, bottom: 48 },
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
        Rect { left: 128, top: 32, right: 144, bottom: 48 },
        Rect { left: 144, top: 32, right: 160, bottom: 48 },
        Rect { left: 48, top: 32, right: 64, bottom: 48 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
        Rect { left: 144, top: 48, right: 160, bottom: 64 },
    ])
}

fn default_n253_experience_capsule() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 0, top: 64, right: 16, bottom: 80 }, Rect { left: 16, top: 64, right: 32, bottom: 80 }])
}

fn default_n254_helicopter() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 0, top: 0, right: 128, bottom: 64 }, Rect { left: 0, top: 64, right: 128, bottom: 128 }])
}

fn default_n255_helicopter_blades() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 240, bottom: 16 },
        Rect { left: 128, top: 16, right: 240, bottom: 32 },
        Rect { left: 128, top: 32, right: 240, bottom: 48 },
        Rect { left: 128, top: 16, right: 240, bottom: 32 },
        Rect { left: 240, top: 0, right: 320, bottom: 16 },
        Rect { left: 240, top: 16, right: 320, bottom: 32 },
        Rect { left: 240, top: 32, right: 320, bottom: 48 },
        Rect { left: 240, top: 16, right: 320, bottom: 32 },
    ])
}

fn default_n256_doctor_facing_away() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 48, top: 160, right: 72, bottom: 192 },
        Rect { left: 72, top: 160, right: 96, bottom: 192 },
        Rect { left: 0, top: 128, right: 24, bottom: 160 },
        Rect { left: 24, top: 128, right: 48, bottom: 160 },
        Rect { left: 0, top: 160, right: 24, bottom: 192 },
        Rect { left: 24, top: 160, right: 48, bottom: 192 },
    ])
}

fn default_n257_red_crystal() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 176, top: 32, right: 184, bottom: 48 },
        Rect { left: 184, top: 32, right: 192, bottom: 48 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
    ])
}

fn default_n258_mimiga_sleeping() -> Rect<u16> {
    Rect { left: 48, top: 32, right: 64, bottom: 48 }
}

fn default_n259_curly_unconscious() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 224, top: 96, right: 240, bottom: 112 },
        Rect { left: 224, top: 112, right: 240, bottom: 128 },
    ])
}

fn default_n260_shovel_brigade_caged() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 128, top: 64, right: 144, bottom: 80 },
        Rect { left: 144, top: 64, right: 160, bottom: 80 },
        Rect { left: 224, top: 64, right: 240, bottom: 80 },
        Rect { left: 128, top: 80, right: 144, bottom: 96 },
        Rect { left: 144, top: 80, right: 160, bottom: 96 },
        Rect { left: 224, top: 80, right: 240, bottom: 96 },
    ])
}

fn default_n261_chie_caged() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
        Rect { left: 128, top: 32, right: 144, bottom: 48 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
    ])
}

fn default_n262_chaco_caged() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
    ])
}

fn default_n263_doctor_boss() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 24, bottom: 32 },
        Rect { left: 24, top: 0, right: 48, bottom: 32 },
        Rect { left: 48, top: 0, right: 72, bottom: 32 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 72, top: 0, right: 96, bottom: 32 },
        Rect { left: 96, top: 0, right: 120, bottom: 32 },
        Rect { left: 120, top: 0, right: 144, bottom: 32 },
        Rect { left: 144, top: 0, right: 168, bottom: 32 },
        Rect { left: 264, top: 0, right: 288, bottom: 32 },
        Rect { left: 0, top: 32, right: 24, bottom: 64 },
        Rect { left: 24, top: 32, right: 48, bottom: 64 },
        Rect { left: 48, top: 32, right: 72, bottom: 64 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 72, top: 32, right: 96, bottom: 64 },
        Rect { left: 96, top: 32, right: 120, bottom: 64 },
        Rect { left: 120, top: 32, right: 144, bottom: 64 },
        Rect { left: 144, top: 32, right: 168, bottom: 64 },
        Rect { left: 264, top: 32, right: 288, bottom: 64 },
    ])
}

fn default_n264_doctor_boss_red_projectile() -> Rect<u16> {
    Rect { left: 288, top: 0, right: 304, bottom: 16 }
}

fn default_n265_doctor_boss_red_projectile_trail() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 288, top: 16, right: 304, bottom: 32 },
        Rect { left: 288, top: 32, right: 304, bottom: 48 },
        Rect { left: 288, top: 48, right: 304, bottom: 64 },
    ])
}

fn default_n266_doctor_boss_red_projectile_bouncing() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 304, top: 16, right: 320, bottom: 32 },
        Rect { left: 304, top: 32, right: 320, bottom: 48 },
    ])
}

fn default_n267_muscle_doctor() -> SafeNPCRect<20> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 0, top: 64, right: 40, bottom: 112 },
        Rect { left: 40, top: 64, right: 80, bottom: 112 },
        Rect { left: 80, top: 64, right: 120, bottom: 112 },
        Rect { left: 120, top: 64, right: 160, bottom: 112 },
        Rect { left: 160, top: 64, right: 200, bottom: 112 },
        Rect { left: 200, top: 64, right: 240, bottom: 112 },
        Rect { left: 240, top: 64, right: 280, bottom: 112 },
        Rect { left: 280, top: 64, right: 320, bottom: 112 },
        Rect { left: 0, top: 160, right: 40, bottom: 208 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 0, top: 112, right: 40, bottom: 160 },
        Rect { left: 40, top: 112, right: 80, bottom: 160 },
        Rect { left: 80, top: 112, right: 120, bottom: 160 },
        Rect { left: 120, top: 112, right: 160, bottom: 160 },
        Rect { left: 160, top: 112, right: 200, bottom: 160 },
        Rect { left: 200, top: 112, right: 240, bottom: 160 },
        Rect { left: 240, top: 112, right: 280, bottom: 160 },
        Rect { left: 280, top: 112, right: 320, bottom: 160 },
        Rect { left: 40, top: 160, right: 80, bottom: 208 },
    ])
}

fn default_n268_igor_enemy() -> SafeNPCRect<20> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 40, top: 0, right: 80, bottom: 40 },
        Rect { left: 80, top: 0, right: 120, bottom: 40 },
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 120, top: 0, right: 160, bottom: 40 },
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 40, top: 80, right: 80, bottom: 120 },
        Rect { left: 0, top: 80, right: 40, bottom: 120 },
        Rect { left: 240, top: 0, right: 280, bottom: 40 },
        Rect { left: 280, top: 0, right: 320, bottom: 40 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 40, top: 40, right: 80, bottom: 80 },
        Rect { left: 80, top: 40, right: 120, bottom: 80 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 120, top: 40, right: 160, bottom: 80 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 160, top: 80, right: 200, bottom: 120 },
        Rect { left: 120, top: 80, right: 160, bottom: 120 },
        Rect { left: 240, top: 40, right: 280, bottom: 80 },
        Rect { left: 280, top: 40, right: 320, bottom: 80 },
    ])
}

fn default_n269_red_bat_bouncing() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 232, top: 0, right: 248, bottom: 16 },
        Rect { left: 248, top: 0, right: 264, bottom: 16 },
        Rect { left: 248, top: 16, right: 264, bottom: 32 },
        Rect { left: 232, top: 32, right: 248, bottom: 48 },
        Rect { left: 248, top: 32, right: 264, bottom: 48 },
        Rect { left: 248, top: 48, right: 264, bottom: 64 },
    ])
}

fn default_n270_doctor_red_energy() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 170, top: 34, right: 174, bottom: 38 },
        Rect { left: 170, top: 42, right: 174, bottom: 46 },
    ])
}

fn default_n273_droll_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 248, top: 40, right: 272, bottom: 64 },
        Rect { left: 272, top: 40, right: 296, bottom: 64 },
        Rect { left: 296, top: 40, right: 320, bottom: 64 },
    ])
}

fn default_n274_droll() -> SafeNPCRect<12> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 32, bottom: 40 },
        Rect { left: 32, top: 0, right: 64, bottom: 40 },
        Rect { left: 64, top: 0, right: 96, bottom: 40 },
        Rect { left: 64, top: 80, right: 96, bottom: 120 },
        Rect { left: 96, top: 80, right: 128, bottom: 120 },
        Rect { left: 96, top: 0, right: 128, bottom: 40 },
        Rect { left: 0, top: 40, right: 32, bottom: 80 },
        Rect { left: 32, top: 40, right: 64, bottom: 80 },
        Rect { left: 64, top: 40, right: 96, bottom: 80 },
        Rect { left: 64, top: 120, right: 96, bottom: 160 },
        Rect { left: 96, top: 120, right: 128, bottom: 160 },
        Rect { left: 96, top: 40, right: 128, bottom: 80 },
    ])
}

fn default_n275_puppy_plantation() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 272, top: 80, right: 288, bottom: 96 },
        Rect { left: 288, top: 80, right: 304, bottom: 96 },
        Rect { left: 272, top: 80, right: 288, bottom: 96 },
        Rect { left: 304, top: 80, right: 320, bottom: 96 },
    ])
}

fn default_n276_red_demon() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 32, bottom: 104 },
        Rect { left: 32, top: 64, right: 64, bottom: 104 },
        Rect { left: 64, top: 64, right: 96, bottom: 104 },
        Rect { left: 96, top: 64, right: 128, bottom: 104 },
        Rect { left: 128, top: 64, right: 160, bottom: 104 },
        Rect { left: 160, top: 64, right: 192, bottom: 104 },
        Rect { left: 192, top: 64, right: 224, bottom: 104 },
        Rect { left: 224, top: 64, right: 256, bottom: 104 },
        Rect { left: 256, top: 64, right: 288, bottom: 104 },
        Rect { left: 0, top: 104, right: 32, bottom: 144 },
        Rect { left: 32, top: 104, right: 64, bottom: 144 },
        Rect { left: 64, top: 104, right: 96, bottom: 144 },
        Rect { left: 96, top: 104, right: 128, bottom: 144 },
        Rect { left: 128, top: 104, right: 160, bottom: 144 },
        Rect { left: 160, top: 104, right: 192, bottom: 144 },
        Rect { left: 192, top: 104, right: 224, bottom: 144 },
        Rect { left: 224, top: 104, right: 256, bottom: 144 },
        Rect { left: 256, top: 104, right: 288, bottom: 144 },
    ])
}

fn default_n277_red_demon_projectile() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 152, bottom: 24 },
        Rect { left: 152, top: 0, right: 176, bottom: 24 },
        Rect { left: 176, top: 0, right: 200, bottom: 24 },
    ])
}

fn default_n278_little_family() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 0, top: 120, right: 8, bottom: 128 },
        Rect { left: 8, top: 120, right: 16, bottom: 128 },
        Rect { left: 16, top: 120, right: 24, bottom: 128 },
        Rect { left: 24, top: 120, right: 32, bottom: 128 },
        Rect { left: 32, top: 120, right: 40, bottom: 128 },
        Rect { left: 40, top: 120, right: 48, bottom: 128 },
    ])
}

fn default_n279_large_falling_block() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 0, top: 16, right: 32, bottom: 48 }, Rect { left: 16, top: 0, right: 32, bottom: 16 }])
}

fn default_n280_sue_teleported() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
        Rect { left: 144, top: 32, right: 160, bottom: 48 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 144, top: 48, right: 160, bottom: 64 },
    ])
}

fn default_n282_mini_undead_core_active() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 256, top: 80, right: 320, bottom: 120 },
        Rect { left: 256, top: 0, right: 320, bottom: 40 },
        Rect { left: 256, top: 120, right: 320, bottom: 160 },
    ])
}

fn default_n283_misery_possessed() -> SafeNPCRect<22> {
    SafeNPCRect([
        Rect { left: 0, top: 64, right: 32, bottom: 96 },
        Rect { left: 32, top: 64, right: 64, bottom: 96 },
        Rect { left: 64, top: 64, right: 96, bottom: 96 },
        Rect { left: 96, top: 64, right: 128, bottom: 96 },
        Rect { left: 128, top: 64, right: 160, bottom: 96 },
        Rect { left: 160, top: 64, right: 192, bottom: 96 },
        Rect { left: 192, top: 64, right: 224, bottom: 96 },
        Rect { left: 224, top: 64, right: 256, bottom: 96 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 256, top: 64, right: 288, bottom: 96 },
        Rect { left: 288, top: 64, right: 320, bottom: 96 },
        Rect { left: 0, top: 96, right: 32, bottom: 128 },
        Rect { left: 32, top: 96, right: 64, bottom: 128 },
        Rect { left: 64, top: 96, right: 96, bottom: 128 },
        Rect { left: 96, top: 96, right: 128, bottom: 128 },
        Rect { left: 128, top: 96, right: 160, bottom: 128 },
        Rect { left: 160, top: 96, right: 192, bottom: 128 },
        Rect { left: 192, top: 96, right: 224, bottom: 128 },
        Rect { left: 224, top: 96, right: 256, bottom: 128 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 256, top: 96, right: 288, bottom: 128 },
        Rect { left: 288, top: 96, right: 320, bottom: 128 },
    ])
}

fn default_n284_sue_possessed() -> SafeNPCRect<26> {
    SafeNPCRect([
        Rect { left: 0, top: 128, right: 32, bottom: 160 },
        Rect { left: 32, top: 128, right: 64, bottom: 160 },
        Rect { left: 64, top: 128, right: 96, bottom: 160 },
        Rect { left: 96, top: 128, right: 128, bottom: 160 },
        Rect { left: 128, top: 128, right: 160, bottom: 160 },
        Rect { left: 160, top: 128, right: 192, bottom: 160 },
        Rect { left: 192, top: 128, right: 224, bottom: 160 },
        Rect { left: 224, top: 128, right: 256, bottom: 160 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 256, top: 128, right: 288, bottom: 160 },
        Rect { left: 288, top: 128, right: 320, bottom: 160 },
        Rect { left: 224, top: 64, right: 256, bottom: 96 },
        Rect { left: 208, top: 32, right: 224, bottom: 48 },
        Rect { left: 0, top: 160, right: 32, bottom: 192 },
        Rect { left: 32, top: 160, right: 64, bottom: 192 },
        Rect { left: 64, top: 160, right: 96, bottom: 192 },
        Rect { left: 96, top: 160, right: 128, bottom: 192 },
        Rect { left: 128, top: 160, right: 160, bottom: 192 },
        Rect { left: 160, top: 160, right: 192, bottom: 192 },
        Rect { left: 192, top: 160, right: 224, bottom: 192 },
        Rect { left: 224, top: 160, right: 256, bottom: 192 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 256, top: 160, right: 288, bottom: 192 },
        Rect { left: 288, top: 160, right: 320, bottom: 192 },
        Rect { left: 224, top: 96, right: 256, bottom: 128 },
        Rect { left: 208, top: 48, right: 224, bottom: 64 },
    ])
}

fn default_n285_undead_core_spiral_projectile() -> Rect<u16> {
    Rect { left: 232, top: 104, right: 248, bottom: 120 }
}

fn default_n286_undead_core_spiral_projectile_trail() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 232, top: 120, right: 248, bottom: 136 },
        Rect { left: 232, top: 136, right: 248, bottom: 152 },
        Rect { left: 232, top: 152, right: 248, bottom: 168 },
    ])
}

fn default_n287_orange_smoke() -> SafeNPCRect<7> {
    SafeNPCRect([
        Rect { left: 0, top: 224, right: 16, bottom: 240 },
        Rect { left: 16, top: 224, right: 32, bottom: 240 },
        Rect { left: 32, top: 224, right: 48, bottom: 240 },
        Rect { left: 48, top: 224, right: 64, bottom: 240 },
        Rect { left: 64, top: 224, right: 80, bottom: 240 },
        Rect { left: 80, top: 224, right: 96, bottom: 240 },
        Rect { left: 96, top: 224, right: 112, bottom: 240 },
    ])
}

fn default_n288_undead_core_exploding_rock() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 232, top: 72, right: 248, bottom: 88 },
        Rect { left: 232, top: 88, right: 248, bottom: 104 },
        Rect { left: 232, top: 0, right: 256, bottom: 24 },
        Rect { left: 232, top: 24, right: 256, bottom: 48 },
        Rect { left: 232, top: 48, right: 256, bottom: 72 },
    ])
}

fn default_n289_critter_orange() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 160, top: 32, right: 176, bottom: 48 },
        Rect { left: 176, top: 32, right: 192, bottom: 48 },
        Rect { left: 192, top: 32, right: 208, bottom: 48 },
        Rect { left: 160, top: 48, right: 176, bottom: 64 },
        Rect { left: 176, top: 48, right: 192, bottom: 64 },
        Rect { left: 192, top: 48, right: 208, bottom: 64 },
    ])
}

fn default_n290_bat_misery() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 112, top: 32, right: 128, bottom: 48 },
        Rect { left: 128, top: 32, right: 144, bottom: 48 },
        Rect { left: 144, top: 32, right: 160, bottom: 48 },
        Rect { left: 112, top: 48, right: 128, bottom: 64 },
        Rect { left: 128, top: 48, right: 144, bottom: 64 },
        Rect { left: 144, top: 48, right: 160, bottom: 64 },
    ])
}

fn default_n291_mini_undead_core_inactive() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 256, top: 80, right: 320, bottom: 120 },
        Rect { left: 256, top: 0, right: 320, bottom: 40 },
    ])
}

fn default_n293_undead_core_energy_shot() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 240, top: 200, right: 280, bottom: 240 },
        Rect { left: 280, top: 200, right: 320, bottom: 240 },
    ])
}

fn default_n295_cloud() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 208, bottom: 64 },
        Rect { left: 32, top: 64, right: 144, bottom: 96 },
        Rect { left: 32, top: 96, right: 104, bottom: 128 },
        Rect { left: 104, top: 96, right: 144, bottom: 128 },
    ])
}

fn default_n297_sue_dragon_mouth() -> Rect<u16> {
    Rect { left: 112, top: 48, right: 128, bottom: 64 }
}

fn default_n298_intro_doctor() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 72, top: 128, right: 88, bottom: 160 },
        Rect { left: 88, top: 128, right: 104, bottom: 160 },
        Rect { left: 104, top: 128, right: 120, bottom: 160 },
        Rect { left: 72, top: 128, right: 88, bottom: 160 },
        Rect { left: 120, top: 128, right: 136, bottom: 160 },
        Rect { left: 72, top: 128, right: 88, bottom: 160 },
        Rect { left: 104, top: 160, right: 120, bottom: 192 },
        Rect { left: 120, top: 160, right: 136, bottom: 192 },
    ])
}

fn default_n299_intro_balrog_misery() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 0, top: 0, right: 48, bottom: 48 }, Rect { left: 48, top: 0, right: 96, bottom: 48 }])
}

fn default_n300_intro_demon_crown() -> Rect<u16> {
    Rect { left: 192, top: 80, right: 208, bottom: 96 }
}

fn default_n301_misery_fish_missile() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
        Rect { left: 192, top: 0, right: 208, bottom: 16 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 192, top: 16, right: 208, bottom: 32 },
    ])
}

fn default_n303_curly_machine_gun() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 216, top: 152, right: 232, bottom: 168 },
        Rect { left: 232, top: 152, right: 248, bottom: 168 },
        Rect { left: 216, top: 168, right: 232, bottom: 184 },
        Rect { left: 232, top: 168, right: 248, bottom: 184 },
    ])
}

fn default_n304_gaudi_hospital() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 176, right: 24, bottom: 192 },
        Rect { left: 24, top: 176, right: 48, bottom: 192 },
        Rect { left: 48, top: 176, right: 72, bottom: 192 },
        Rect { left: 72, top: 176, right: 96, bottom: 192 },
    ])
}

fn default_n305_small_puppy() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 160, top: 144, right: 176, bottom: 160 },
        Rect { left: 176, top: 144, right: 192, bottom: 160 },
        Rect { left: 160, top: 160, right: 176, bottom: 176 },
        Rect { left: 176, top: 160, right: 192, bottom: 176 },
    ])
}

fn default_n306_balrog_nurse() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 240, top: 96, right: 280, bottom: 128 },
        Rect { left: 280, top: 96, right: 320, bottom: 128 },
        Rect { left: 160, top: 152, right: 200, bottom: 184 },
        Rect { left: 200, top: 152, right: 240, bottom: 184 },
    ])
}

fn default_n307_santa_caged() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 32, right: 16, bottom: 48 },
        Rect { left: 16, top: 32, right: 32, bottom: 48 },
        Rect { left: 0, top: 48, right: 16, bottom: 64 },
        Rect { left: 16, top: 48, right: 32, bottom: 64 },
    ])
}

fn default_n308_stumpy() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 128, top: 112, right: 144, bottom: 128 },
        Rect { left: 144, top: 112, right: 160, bottom: 128 },
        Rect { left: 128, top: 128, right: 144, bottom: 144 },
        Rect { left: 144, top: 128, right: 160, bottom: 144 },
    ])
}

fn default_n309_bute() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 16, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
    ])
}

fn default_n310_bute_sword() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 32, top: 0, right: 56, bottom: 16 },
        Rect { left: 56, top: 0, right: 80, bottom: 16 },
        Rect { left: 80, top: 0, right: 104, bottom: 16 },
        Rect { left: 104, top: 0, right: 128, bottom: 16 },
        Rect { left: 128, top: 0, right: 152, bottom: 16 },
        Rect { left: 32, top: 16, right: 56, bottom: 32 },
        Rect { left: 56, top: 16, right: 80, bottom: 32 },
        Rect { left: 80, top: 16, right: 104, bottom: 32 },
        Rect { left: 104, top: 16, right: 128, bottom: 32 },
        Rect { left: 128, top: 16, right: 152, bottom: 32 },
    ])
}

fn default_n311_bute_archer() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 0, top: 32, right: 24, bottom: 56 },
        Rect { left: 24, top: 32, right: 48, bottom: 56 },
        Rect { left: 48, top: 32, right: 72, bottom: 56 },
        Rect { left: 72, top: 32, right: 96, bottom: 56 },
        Rect { left: 96, top: 32, right: 120, bottom: 56 },
        Rect { left: 120, top: 32, right: 144, bottom: 56 },
        Rect { left: 144, top: 32, right: 168, bottom: 56 },
        Rect { left: 0, top: 56, right: 24, bottom: 80 },
        Rect { left: 24, top: 56, right: 48, bottom: 80 },
        Rect { left: 48, top: 56, right: 72, bottom: 80 },
        Rect { left: 72, top: 56, right: 96, bottom: 80 },
        Rect { left: 96, top: 56, right: 120, bottom: 80 },
        Rect { left: 120, top: 56, right: 144, bottom: 80 },
        Rect { left: 144, top: 56, right: 168, bottom: 80 },
    ])
}

fn default_n312_bute_arrow_projectile() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 0, top: 160, right: 16, bottom: 176 },
        Rect { left: 16, top: 160, right: 32, bottom: 176 },
        Rect { left: 32, top: 160, right: 48, bottom: 176 },
        Rect { left: 48, top: 160, right: 64, bottom: 176 },
        Rect { left: 64, top: 160, right: 80, bottom: 176 },
        Rect { left: 0, top: 176, right: 16, bottom: 192 },
        Rect { left: 16, top: 176, right: 32, bottom: 192 },
        Rect { left: 32, top: 176, right: 48, bottom: 192 },
        Rect { left: 48, top: 176, right: 64, bottom: 192 },
        Rect { left: 64, top: 176, right: 80, bottom: 192 },
    ])
}

fn default_n313_ma_pignon() -> SafeNPCRect<28> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 144, top: 0, right: 160, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
        Rect { left: 192, top: 0, right: 208, bottom: 16 },
        Rect { left: 208, top: 0, right: 224, bottom: 16 },
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
        Rect { left: 240, top: 0, right: 256, bottom: 16 },
        Rect { left: 256, top: 0, right: 272, bottom: 16 },
        Rect { left: 272, top: 0, right: 288, bottom: 16 },
        Rect { left: 288, top: 0, right: 304, bottom: 16 },
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
        Rect { left: 304, top: 0, right: 320, bottom: 16 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 144, top: 16, right: 160, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 192, top: 16, right: 208, bottom: 32 },
        Rect { left: 208, top: 16, right: 224, bottom: 32 },
        Rect { left: 224, top: 16, right: 240, bottom: 32 },
        Rect { left: 240, top: 16, right: 256, bottom: 32 },
        Rect { left: 256, top: 16, right: 272, bottom: 32 },
        Rect { left: 272, top: 16, right: 288, bottom: 32 },
        Rect { left: 288, top: 16, right: 304, bottom: 32 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 304, top: 16, right: 320, bottom: 32 },
    ])
}

fn default_n314_ma_pignon_rock() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 64, top: 64, right: 80, bottom: 80 },
        Rect { left: 80, top: 64, right: 96, bottom: 80 },
        Rect { left: 96, top: 64, right: 112, bottom: 80 },
    ])
}

fn default_n315_ma_pignon_clone() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 128, top: 0, right: 144, bottom: 16 },
        Rect { left: 160, top: 0, right: 176, bottom: 16 },
        Rect { left: 176, top: 0, right: 192, bottom: 16 },
        Rect { left: 192, top: 0, right: 208, bottom: 16 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 160, top: 16, right: 176, bottom: 32 },
        Rect { left: 176, top: 16, right: 192, bottom: 32 },
        Rect { left: 192, top: 16, right: 208, bottom: 32 },
    ])
}

fn default_n316_bute_dead() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 248, top: 32, right: 272, bottom: 56 },
        Rect { left: 272, top: 32, right: 296, bottom: 56 },
        Rect { left: 296, top: 32, right: 320, bottom: 56 },
        Rect { left: 248, top: 56, right: 272, bottom: 80 },
        Rect { left: 272, top: 56, right: 296, bottom: 80 },
        Rect { left: 296, top: 56, right: 320, bottom: 80 },
    ])
}

fn default_n317_mesa() -> SafeNPCRect<8> {
    SafeNPCRect([
        Rect { left: 0, top: 80, right: 32, bottom: 120 },
        Rect { left: 32, top: 80, right: 64, bottom: 120 },
        Rect { left: 64, top: 80, right: 96, bottom: 120 },
        Rect { left: 96, top: 80, right: 128, bottom: 120 },
        Rect { left: 0, top: 120, right: 32, bottom: 160 },
        Rect { left: 32, top: 120, right: 64, bottom: 160 },
        Rect { left: 64, top: 120, right: 96, bottom: 160 },
        Rect { left: 96, top: 120, right: 128, bottom: 160 },
    ])
}

fn default_n318_mesa_dead() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 224, top: 80, right: 256, bottom: 120 },
        Rect { left: 256, top: 80, right: 288, bottom: 120 },
        Rect { left: 288, top: 80, right: 320, bottom: 120 },
        Rect { left: 224, top: 120, right: 256, bottom: 160 },
        Rect { left: 256, top: 120, right: 288, bottom: 160 },
        Rect { left: 288, top: 120, right: 320, bottom: 160 },
    ])
}

fn default_n319_mesa_block() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 16, top: 0, right: 32, bottom: 16 },
        Rect { left: 96, top: 80, right: 112, bottom: 96 },
    ])
}

fn default_n320_curly_carried() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 16, top: 96, right: 32, bottom: 112 },
        Rect { left: 48, top: 96, right: 64, bottom: 112 },
        Rect { left: 96, top: 96, right: 112, bottom: 112 },
        Rect { left: 16, top: 112, right: 32, bottom: 128 },
        Rect { left: 48, top: 112, right: 64, bottom: 128 },
        Rect { left: 96, top: 112, right: 112, bottom: 128 },
    ])
}

fn default_n321_curly_nemesis() -> SafeNPCRect<6> {
    SafeNPCRect([
        Rect { left: 136, top: 152, right: 152, bottom: 168 },
        Rect { left: 152, top: 152, right: 168, bottom: 168 },
        Rect { left: 168, top: 152, right: 184, bottom: 168 },
        Rect { left: 136, top: 168, right: 152, bottom: 184 },
        Rect { left: 152, top: 168, right: 168, bottom: 184 },
        Rect { left: 168, top: 168, right: 184, bottom: 184 },
    ])
}

fn default_n322_deleet() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 272, top: 216, right: 296, bottom: 240 },
        Rect { left: 296, top: 216, right: 320, bottom: 240 },
        Rect { left: 160, top: 216, right: 184, bottom: 240 },
    ])
}

fn default_n323_bute_spinning() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 216, top: 32, right: 232, bottom: 56 },
        Rect { left: 232, top: 32, right: 248, bottom: 56 },
        Rect { left: 216, top: 56, right: 232, bottom: 80 },
        Rect { left: 232, top: 56, right: 248, bottom: 80 },
    ])
}

fn default_n325_heavy_press_lightning() -> SafeNPCRect<7> {
    SafeNPCRect([
        Rect { left: 240, top: 96, right: 272, bottom: 128 },
        Rect { left: 272, top: 96, right: 304, bottom: 128 },
        Rect { left: 240, top: 128, right: 272, bottom: 160 },
        Rect { left: 240, top: 0, right: 256, bottom: 96 },
        Rect { left: 256, top: 0, right: 272, bottom: 96 },
        Rect { left: 272, top: 0, right: 288, bottom: 96 },
        Rect { left: 288, top: 0, right: 304, bottom: 96 },
    ])
}

fn default_n326_sue_itoh_human_transition() -> SafeNPCRect<16> {
    SafeNPCRect([
        Rect { left: 0, top: 128, right: 16, bottom: 152 },
        Rect { left: 16, top: 128, right: 32, bottom: 152 },
        Rect { left: 32, top: 128, right: 48, bottom: 152 },
        Rect { left: 48, top: 128, right: 64, bottom: 152 },
        Rect { left: 64, top: 128, right: 80, bottom: 152 },
        Rect { left: 80, top: 128, right: 96, bottom: 152 },
        Rect { left: 96, top: 128, right: 112, bottom: 152 },
        Rect { left: 112, top: 128, right: 128, bottom: 152 },
        Rect { left: 128, top: 128, right: 144, bottom: 152 },
        Rect { left: 144, top: 128, right: 160, bottom: 152 },
        Rect { left: 160, top: 128, right: 176, bottom: 152 },
        Rect { left: 176, top: 128, right: 192, bottom: 152 },
        Rect { left: 192, top: 128, right: 208, bottom: 152 },
        Rect { left: 208, top: 128, right: 224, bottom: 152 },
        Rect { left: 224, top: 128, right: 240, bottom: 152 },
        Rect { left: 32, top: 152, right: 48, bottom: 176 },
    ])
}

fn default_n327_sneeze() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 240, top: 80, right: 256, bottom: 96 },
        Rect { left: 256, top: 80, right: 272, bottom: 96 },
    ])
}

fn default_n328_human_transform_machine() -> Rect<u16> {
    Rect { left: 96, top: 0, right: 128, bottom: 48 }
}

fn default_n329_laboratory_fan() -> SafeNPCRect<2> {
    SafeNPCRect([Rect { left: 48, top: 0, right: 64, bottom: 16 }, Rect { left: 64, top: 0, right: 80, bottom: 16 }])
}

fn default_n330_rolling() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 144, top: 136, right: 160, bottom: 152 },
        Rect { left: 160, top: 136, right: 176, bottom: 152 },
        Rect { left: 176, top: 136, right: 192, bottom: 152 },
    ])
}

fn default_n331_ballos_bone_projectile() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 288, top: 80, right: 304, bottom: 96 },
        Rect { left: 304, top: 80, right: 320, bottom: 96 },
        Rect { left: 288, top: 96, right: 304, bottom: 112 },
        Rect { left: 304, top: 96, right: 320, bottom: 112 },
    ])
}

fn default_n332_ballos_shockwave() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 144, top: 96, right: 168, bottom: 120 },
        Rect { left: 168, top: 96, right: 192, bottom: 120 },
        Rect { left: 192, top: 96, right: 216, bottom: 120 },
    ])
}

fn default_n333_ballos_lightning() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 80, top: 120, right: 104, bottom: 144 },
        Rect { left: 104, top: 120, right: 128, bottom: 144 },
    ])
}

fn default_n334_sweat() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 160, top: 184, right: 168, bottom: 200 },
        Rect { left: 168, top: 184, right: 176, bottom: 200 },
        Rect { left: 176, top: 184, right: 184, bottom: 200 },
        Rect { left: 184, top: 184, right: 192, bottom: 200 },
    ])
}

fn default_n335_ikachan() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 16, top: 16, right: 32, bottom: 32 },
        Rect { left: 32, top: 16, right: 48, bottom: 32 },
    ])
}

fn default_n337_numahachi() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 256, top: 112, right: 288, bottom: 152 },
        Rect { left: 288, top: 112, right: 320, bottom: 152 },
    ])
}

fn default_n338_green_devil() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 288, top: 0, right: 304, bottom: 16 },
        Rect { left: 304, top: 0, right: 320, bottom: 16 },
        Rect { left: 288, top: 16, right: 304, bottom: 32 },
        Rect { left: 304, top: 16, right: 320, bottom: 32 },
    ])
}

fn default_n340_ballos() -> SafeNPCRect<22> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 48, bottom: 40 },
        Rect { left: 48, top: 0, right: 96, bottom: 40 },
        Rect { left: 96, top: 0, right: 144, bottom: 40 },
        Rect { left: 144, top: 0, right: 192, bottom: 40 },
        Rect { left: 192, top: 0, right: 240, bottom: 40 },
        Rect { left: 240, top: 0, right: 288, bottom: 40 },
        Rect { left: 0, top: 80, right: 48, bottom: 120 },
        Rect { left: 48, top: 80, right: 96, bottom: 120 },
        Rect { left: 96, top: 80, right: 144, bottom: 120 },
        Rect { left: 144, top: 80, right: 192, bottom: 120 },
        Rect { left: 192, top: 80, right: 240, bottom: 120 },
        Rect { left: 0, top: 40, right: 48, bottom: 80 },
        Rect { left: 48, top: 40, right: 96, bottom: 80 },
        Rect { left: 96, top: 40, right: 144, bottom: 80 },
        Rect { left: 144, top: 40, right: 192, bottom: 80 },
        Rect { left: 192, top: 40, right: 240, bottom: 80 },
        Rect { left: 240, top: 40, right: 288, bottom: 80 },
        Rect { left: 0, top: 120, right: 48, bottom: 160 },
        Rect { left: 48, top: 120, right: 96, bottom: 160 },
        Rect { left: 96, top: 120, right: 144, bottom: 160 },
        Rect { left: 144, top: 120, right: 192, bottom: 160 },
        Rect { left: 192, top: 120, right: 240, bottom: 160 },
    ])
}

fn default_n341_ballos_1_head() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 288, top: 32, right: 320, bottom: 48 },
        Rect { left: 288, top: 48, right: 320, bottom: 64 },
        Rect { left: 288, top: 64, right: 320, bottom: 80 },
    ])
}

fn default_n342_ballos_orbiting_eye() -> SafeNPCRect<3> {
    SafeNPCRect([
        Rect { left: 240, top: 48, right: 280, bottom: 88 },
        Rect { left: 240, top: 88, right: 280, bottom: 128 },
        Rect { left: 280, top: 48, right: 320, bottom: 88 },
    ])
}

fn default_n343_ballos_3_cutscene() -> Rect<u16> {
    Rect { left: 0, top: 0, right: 120, bottom: 120 }
}

fn default_n344_ballos_3_eyes() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 272, top: 0, right: 296, bottom: 16 },
        Rect { left: 296, top: 0, right: 320, bottom: 16 },
    ])
}

fn default_n345_ballos_skull_projectile() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 128, top: 176, right: 144, bottom: 192 },
        Rect { left: 144, top: 176, right: 160, bottom: 192 },
        Rect { left: 160, top: 176, right: 176, bottom: 192 },
        Rect { left: 176, top: 176, right: 192, bottom: 192 },
    ])
}

fn default_n346_ballos_orbiting_platform() -> Rect<u16> {
    Rect { left: 240, top: 0, right: 272, bottom: 16 }
}

fn default_n347_hoppy() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 256, top: 48, right: 272, bottom: 64 },
        Rect { left: 272, top: 48, right: 288, bottom: 64 },
        Rect { left: 288, top: 48, right: 304, bottom: 64 },
        Rect { left: 304, top: 48, right: 320, bottom: 64 },
    ])
}

fn default_n348_ballos_4_spikes() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 128, top: 152, right: 160, bottom: 176 },
        Rect { left: 160, top: 152, right: 192, bottom: 176 },
    ])
}

fn default_n349_statue() -> Rect<u16> {
    Rect { left: 0, top: 0, right: 16, bottom: 16 }
}

fn default_n350_flying_bute_archer() -> SafeNPCRect<14> {
    SafeNPCRect([
        Rect { left: 0, top: 160, right: 24, bottom: 184 },
        Rect { left: 24, top: 160, right: 48, bottom: 184 },
        Rect { left: 48, top: 160, right: 72, bottom: 184 },
        Rect { left: 72, top: 160, right: 96, bottom: 184 },
        Rect { left: 96, top: 160, right: 120, bottom: 184 },
        Rect { left: 120, top: 160, right: 144, bottom: 184 },
        Rect { left: 144, top: 160, right: 168, bottom: 184 },
        Rect { left: 0, top: 184, right: 24, bottom: 208 },
        Rect { left: 24, top: 184, right: 48, bottom: 208 },
        Rect { left: 48, top: 184, right: 72, bottom: 208 },
        Rect { left: 72, top: 184, right: 96, bottom: 208 },
        Rect { left: 96, top: 184, right: 120, bottom: 208 },
        Rect { left: 120, top: 184, right: 144, bottom: 208 },
        Rect { left: 144, top: 184, right: 168, bottom: 208 },
    ])
}

fn default_n351_statue_shootable() -> SafeNPCRect<9> {
    SafeNPCRect([
        Rect { left: 0, top: 96, right: 32, bottom: 136 },
        Rect { left: 32, top: 96, right: 64, bottom: 136 },
        Rect { left: 64, top: 96, right: 96, bottom: 136 },
        Rect { left: 96, top: 96, right: 128, bottom: 136 },
        Rect { left: 128, top: 96, right: 160, bottom: 136 },
        Rect { left: 0, top: 176, right: 32, bottom: 216 },
        Rect { left: 32, top: 176, right: 64, bottom: 216 },
        Rect { left: 64, top: 176, right: 96, bottom: 216 },
        Rect { left: 96, top: 176, right: 128, bottom: 216 },
    ])
}

fn default_n352_ending_characters() -> SafeNPCRect<28> {
    SafeNPCRect([
        Rect { left: 304, top: 48, right: 320, bottom: 64 },
        Rect { left: 224, top: 48, right: 240, bottom: 64 },
        Rect { left: 32, top: 80, right: 48, bottom: 96 },
        Rect { left: 0, top: 80, right: 16, bottom: 96 },
        Rect { left: 224, top: 216, right: 240, bottom: 240 },
        Rect { left: 192, top: 216, right: 208, bottom: 240 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 0, top: 16, right: 16, bottom: 32 },
        Rect { left: 112, top: 192, right: 128, bottom: 216 },
        Rect { left: 80, top: 192, right: 96, bottom: 216 },
        Rect { left: 304, top: 0, right: 320, bottom: 16 },
        Rect { left: 224, top: 0, right: 240, bottom: 16 },
        Rect { left: 176, top: 32, right: 192, bottom: 48 },
        Rect { left: 176, top: 32, right: 192, bottom: 48 },
        Rect { left: 240, top: 16, right: 256, bottom: 32 },
        Rect { left: 224, top: 16, right: 240, bottom: 32 },
        Rect { left: 208, top: 16, right: 224, bottom: 32 },
        Rect { left: 192, top: 16, right: 208, bottom: 32 },
        Rect { left: 280, top: 128, right: 320, bottom: 152 },
        Rect { left: 280, top: 152, right: 320, bottom: 176 },
        Rect { left: 32, top: 112, right: 48, bottom: 128 },
        Rect { left: 0, top: 112, right: 16, bottom: 128 },
        Rect { left: 80, top: 0, right: 96, bottom: 16 },
        Rect { left: 112, top: 0, right: 128, bottom: 16 },
        Rect { left: 16, top: 152, right: 32, bottom: 176 },
        Rect { left: 0, top: 152, right: 16, bottom: 176 },
        Rect { left: 48, top: 16, right: 64, bottom: 32 },
        Rect { left: 48, top: 0, right: 64, bottom: 16 },
    ])
}

fn default_n353_bute_sword_flying() -> SafeNPCRect<8> {
    SafeNPCRect([
        // Entering
        Rect { left: 168, top: 160, right: 184, bottom: 184 },
        Rect { left: 184, top: 160, right: 200, bottom: 184 },
        Rect { left: 168, top: 184, right: 184, bottom: 208 },
        Rect { left: 184, top: 184, right: 200, bottom: 208 },
        // Flying
        Rect { left: 200, top: 160, right: 216, bottom: 176 },
        Rect { left: 216, top: 160, right: 232, bottom: 176 },
        Rect { left: 200, top: 176, right: 216, bottom: 192 },
        Rect { left: 216, top: 176, right: 232, bottom: 192 },
    ])
}

fn default_n355_quote_and_curly_on_balrog() -> SafeNPCRect<4> {
    SafeNPCRect([
        Rect { left: 80, top: 16, right: 96, bottom: 32 },
        Rect { left: 80, top: 96, right: 96, bottom: 112 },
        Rect { left: 128, top: 16, right: 144, bottom: 32 },
        Rect { left: 208, top: 96, right: 224, bottom: 112 },
    ])
}

fn default_n356_balrog_rescuing() -> SafeNPCRect<2> {
    SafeNPCRect([
        Rect { left: 240, top: 128, right: 280, bottom: 152 },
        Rect { left: 240, top: 152, right: 280, bottom: 176 },
    ])
}

fn default_n357_puppy_ghost() -> Rect<u16> {
    Rect { left: 224, top: 136, right: 240, bottom: 152 }
}

fn default_n358_misery_credits() -> SafeNPCRect<5> {
    SafeNPCRect([
        Rect { left: 208, top: 8, right: 224, bottom: 32 },
        Rect { left: 224, top: 8, right: 240, bottom: 32 },
        Rect { left: 240, top: 8, right: 256, bottom: 32 },
        Rect { left: 256, top: 8, right: 272, bottom: 32 },
        Rect { left: 272, top: 8, right: 288, bottom: 32 },
    ])
}

fn default_n360_credits_thank_you() -> Rect<u16> {
    Rect { left: 0, top: 176, right: 48, bottom: 184 }
}

fn default_b01_omega() -> SafeNPCRect<10> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 80, bottom: 56 },
        Rect { left: 80, top: 0, right: 160, bottom: 56 },
        Rect { left: 160, top: 0, right: 240, bottom: 56 },
        Rect { left: 80, top: 0, right: 160, bottom: 56 },
        Rect { left: 80, top: 56, right: 104, bottom: 72 },
        Rect { left: 104, top: 56, right: 128, bottom: 72 },
        Rect { left: 0, top: 56, right: 40, bottom: 88 },
        Rect { left: 40, top: 56, right: 80, bottom: 88 },
        Rect { left: 0, top: 88, right: 40, bottom: 120 },
        Rect { left: 40, top: 88, right: 80, bottom: 120 },
    ])
}

fn default_b02_balfrog() -> SafeNPCRect<18> {
    SafeNPCRect([
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 0, top: 48, right: 80, bottom: 112 },
        Rect { left: 0, top: 112, right: 80, bottom: 176 },
        Rect { left: 0, top: 176, right: 80, bottom: 240 },
        Rect { left: 160, top: 48, right: 240, bottom: 112 },
        Rect { left: 160, top: 112, right: 240, bottom: 200 },
        Rect { left: 200, top: 0, right: 240, bottom: 24 },
        Rect { left: 80, top: 0, right: 120, bottom: 24 },
        Rect { left: 120, top: 0, right: 160, bottom: 24 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 80, top: 48, right: 160, bottom: 112 },
        Rect { left: 80, top: 112, right: 160, bottom: 176 },
        Rect { left: 80, top: 176, right: 160, bottom: 240 },
        Rect { left: 240, top: 48, right: 320, bottom: 112 },
        Rect { left: 240, top: 112, right: 320, bottom: 200 },
        Rect { left: 200, top: 24, right: 240, bottom: 48 },
        Rect { left: 80, top: 24, right: 120, bottom: 48 },
        Rect { left: 120, top: 24, right: 160, bottom: 48 },
    ])
}

fn default_b03_monster_x() -> SafeNPCRect<29> {
    SafeNPCRect([
        // face
        Rect { left: 216, top: 0, right: 320, bottom: 48 },
        Rect { left: 216, top: 48, right: 320, bottom: 96 },
        Rect { left: 216, top: 144, right: 320, bottom: 192 },
        // tracks up
        Rect { left: 0, top: 0, right: 72, bottom: 32 },
        Rect { left: 0, top: 32, right: 72, bottom: 64 },
        Rect { left: 72, top: 0, right: 144, bottom: 32 },
        Rect { left: 144, top: 0, right: 216, bottom: 32 },
        Rect { left: 72, top: 32, right: 144, bottom: 64 },
        Rect { left: 144, top: 32, right: 216, bottom: 64 },
        // tracks down
        Rect { left: 0, top: 64, right: 72, bottom: 96 },
        Rect { left: 0, top: 96, right: 72, bottom: 128 },
        Rect { left: 72, top: 64, right: 144, bottom: 96 },
        Rect { left: 144, top: 64, right: 216, bottom: 96 },
        Rect { left: 72, top: 96, right: 144, bottom: 128 },
        Rect { left: 144, top: 96, right: 216, bottom: 128 },
        // frame
        Rect { left: 0, top: 128, right: 72, bottom: 160 },
        Rect { left: 72, top: 128, right: 144, bottom: 160 },
        Rect { left: 0, top: 160, right: 72, bottom: 192 },
        Rect { left: 72, top: 160, right: 144, bottom: 192 },
        // shield left
        Rect { left: 216, top: 96, right: 264, bottom: 144 },
        // shield right
        Rect { left: 264, top: 96, right: 312, bottom: 144 },
        // part 4
        Rect { left: 0, top: 192, right: 16, bottom: 208 },
        Rect { left: 16, top: 192, right: 32, bottom: 208 },
        Rect { left: 32, top: 192, right: 48, bottom: 208 },
        Rect { left: 48, top: 192, right: 64, bottom: 208 },
        Rect { left: 0, top: 208, right: 16, bottom: 224 },
        Rect { left: 16, top: 208, right: 32, bottom: 224 },
        Rect { left: 32, top: 208, right: 48, bottom: 224 },
        Rect { left: 48, top: 208, right: 64, bottom: 224 },
    ])
}

fn default_b04_core() -> SafeNPCRect<10> {
    SafeNPCRect([
        // face
        Rect { left: 0, top: 0, right: 72, bottom: 112 },
        Rect { left: 0, top: 112, right: 72, bottom: 224 },
        Rect { left: 160, top: 0, right: 232, bottom: 112 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        // tail
        Rect { left: 72, top: 0, right: 160, bottom: 112 },
        Rect { left: 72, top: 112, right: 160, bottom: 224 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        // small head
        Rect { left: 256, top: 0, right: 320, bottom: 40 },
        Rect { left: 256, top: 40, right: 320, bottom: 80 },
        Rect { left: 256, top: 80, right: 320, bottom: 120 },
    ])
}

fn default_b05_ironhead() -> SafeNPCRect<18> {
    SafeNPCRect([
        // set 1
        Rect { left: 0, top: 0, right: 64, bottom: 24 },
        Rect { left: 64, top: 0, right: 128, bottom: 24 },
        Rect { left: 128, top: 0, right: 192, bottom: 24 },
        Rect { left: 64, top: 0, right: 128, bottom: 24 },
        Rect { left: 0, top: 0, right: 64, bottom: 24 },
        Rect { left: 192, top: 0, right: 256, bottom: 24 },
        Rect { left: 256, top: 0, right: 320, bottom: 24 },
        Rect { left: 192, top: 0, right: 256, bottom: 24 },
        Rect { left: 256, top: 48, right: 320, bottom: 72 },
        // set 2
        Rect { left: 0, top: 24, right: 64, bottom: 48 },
        Rect { left: 64, top: 24, right: 128, bottom: 48 },
        Rect { left: 128, top: 24, right: 192, bottom: 48 },
        Rect { left: 64, top: 24, right: 128, bottom: 48 },
        Rect { left: 0, top: 24, right: 64, bottom: 48 },
        Rect { left: 192, top: 24, right: 256, bottom: 48 },
        Rect { left: 256, top: 24, right: 320, bottom: 48 },
        Rect { left: 192, top: 24, right: 256, bottom: 48 },
        Rect { left: 256, top: 48, right: 320, bottom: 72 },
    ])
}

fn default_b06_sisters() -> SafeNPCRect<14> {
    SafeNPCRect([
        // head
        Rect { left: 0, top: 80, right: 40, bottom: 112 },
        Rect { left: 40, top: 80, right: 80, bottom: 112 },
        Rect { left: 80, top: 80, right: 120, bottom: 112 },
        Rect { left: 120, top: 80, right: 160, bottom: 112 },
        Rect { left: 0, top: 112, right: 40, bottom: 144 },
        Rect { left: 40, top: 112, right: 80, bottom: 144 },
        Rect { left: 80, top: 112, right: 120, bottom: 144 },
        Rect { left: 120, top: 112, right: 160, bottom: 144 },
        // body
        Rect { left: 0, top: 0, right: 40, bottom: 40 },
        Rect { left: 40, top: 0, right: 80, bottom: 40 },
        Rect { left: 80, top: 0, right: 120, bottom: 40 },
        Rect { left: 0, top: 40, right: 40, bottom: 80 },
        Rect { left: 40, top: 40, right: 80, bottom: 80 },
        Rect { left: 80, top: 40, right: 120, bottom: 80 },
    ])
}

fn default_b07_undead_core() -> SafeNPCRect<15> {
    SafeNPCRect([
        // face
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 160, top: 112, right: 232, bottom: 152 },
        Rect { left: 160, top: 152, right: 232, bottom: 192 },
        Rect { left: 160, top: 192, right: 232, bottom: 232 },
        Rect { left: 248, top: 160, right: 320, bottom: 200 },
        // head
        Rect { left: 0, top: 0, right: 72, bottom: 112 },
        Rect { left: 0, top: 112, right: 72, bottom: 224 },
        Rect { left: 160, top: 0, right: 232, bottom: 112 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        // tail
        Rect { left: 72, top: 0, right: 160, bottom: 112 },
        Rect { left: 72, top: 112, right: 160, bottom: 224 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        // small head
        Rect { left: 256, top: 0, right: 320, bottom: 40 },
        Rect { left: 256, top: 40, right: 320, bottom: 80 },
        Rect { left: 256, top: 80, right: 320, bottom: 120 },
    ])
}

fn default_b08_heavy_press() -> SafeNPCRect<6> {
    SafeNPCRect([
        // Normal
        Rect { left: 0, top: 0, right: 80, bottom: 120 },
        Rect { left: 80, top: 0, right: 160, bottom: 120 },
        Rect { left: 160, top: 0, right: 240, bottom: 120 },
        // Hurt
        Rect { left: 0, top: 120, right: 80, bottom: 240 },
        Rect { left: 80, top: 120, right: 160, bottom: 240 },
        Rect { left: 160, top: 120, right: 240, bottom: 240 },
    ])
}

fn default_b09_ballos() -> SafeNPCRect<14> {
    SafeNPCRect([
        // B A L L S
        Rect { left: 0, top: 0, right: 120, bottom: 120 },
        Rect { left: 120, top: 0, right: 240, bottom: 120 },
        Rect { left: 0, top: 120, right: 120, bottom: 240 },
        Rect { left: 120, top: 120, right: 240, bottom: 240 },
        // Eye
        Rect { left: 272, top: 0, right: 296, bottom: 16 },
        Rect { left: 272, top: 16, right: 296, bottom: 32 },
        Rect { left: 272, top: 32, right: 296, bottom: 48 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 240, top: 16, right: 264, bottom: 32 },
        Rect { left: 296, top: 0, right: 320, bottom: 16 },
        Rect { left: 296, top: 16, right: 320, bottom: 32 },
        Rect { left: 296, top: 32, right: 320, bottom: 48 },
        Rect { left: 0, top: 0, right: 0, bottom: 0 },
        Rect { left: 240, top: 32, right: 264, bottom: 48 },
    ])
}
