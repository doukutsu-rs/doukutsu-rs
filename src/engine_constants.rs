use std::collections::HashMap;

use log::info;
use maplit::hashmap;

use crate::common::{Direction, Rect};
use crate::str;
use crate::text_script::TextScriptEncoding;

#[derive(Debug, Copy, Clone)]
pub struct PhysicsConsts {
    pub max_dash: isize,
    pub max_move: isize,
    pub gravity_ground: isize,
    pub gravity_air: isize,
    pub dash_ground: isize,
    pub dash_air: isize,
    pub resist: isize,
    pub jump: isize,
}


#[derive(Debug, Copy, Clone)]
pub struct BoosterConsts {
    pub fuel: usize,
    pub b2_0_up: isize,
    pub b2_0_up_nokey: isize,
    pub b2_0_down: isize,
    pub b2_0_left: isize,
    pub b2_0_right: isize,
}

#[derive(Debug, Copy, Clone)]
pub struct MyCharConsts {
    pub cond: u16,
    pub flags: u32,
    pub equip: u16,
    pub direction: Direction,
    pub view: Rect<usize>,
    pub hit: Rect<usize>,
    pub life: usize,
    pub max_life: usize,
    pub unit: u8,
    pub air_physics: PhysicsConsts,
    pub water_physics: PhysicsConsts,
    pub animations_left: [Rect<usize>; 12],
    pub animations_right: [Rect<usize>; 12],
}

#[derive(Debug)]
pub struct CaretConsts {
    pub offsets: [(isize, isize); 18],
    pub bubble_left_rects: Vec<Rect<usize>>,
    pub bubble_right_rects: Vec<Rect<usize>>,
    pub little_particles_rects: Vec<Rect<usize>>,
    pub exhaust_rects: Vec<Rect<usize>>,
    pub question_left_rect: Rect<usize>,
    pub question_right_rect: Rect<usize>,
}

impl Clone for CaretConsts {
    fn clone(&self) -> Self {
        Self {
            offsets: self.offsets,
            bubble_left_rects: self.bubble_left_rects.clone(),
            bubble_right_rects: self.bubble_right_rects.clone(),
            little_particles_rects: self.little_particles_rects.clone(),
            exhaust_rects: self.exhaust_rects.clone(),
            question_left_rect: self.question_left_rect,
            question_right_rect: self.question_right_rect,
        }
    }
}

#[derive(Debug)]
pub struct WorldConsts {
    pub snack_rect: Rect<usize>,
}

impl Clone for WorldConsts {
    fn clone(&self) -> Self {
        Self {
            snack_rect: self.snack_rect,
        }
    }
}

#[derive(Debug)]
pub struct TextScriptConsts {
    pub encoding: TextScriptEncoding,
    pub textbox_rect_top: Rect<usize>,
    pub textbox_rect_middle: Rect<usize>,
    pub textbox_rect_bottom: Rect<usize>,
}

impl Clone for TextScriptConsts {
    fn clone(&self) -> Self {
        Self {
            encoding: self.encoding,
            textbox_rect_top: self.textbox_rect_top,
            textbox_rect_middle: self.textbox_rect_middle,
            textbox_rect_bottom: self.textbox_rect_bottom,
        }
    }
}

#[derive(Debug)]
pub struct EngineConstants {
    pub is_cs_plus: bool,
    pub my_char: MyCharConsts,
    pub booster: BoosterConsts,
    pub caret: CaretConsts,
    pub world: WorldConsts,
    pub tex_sizes: HashMap<String, (usize, usize)>,
    pub textscript: TextScriptConsts,
}

impl Clone for EngineConstants {
    fn clone(&self) -> Self {
        Self {
            is_cs_plus: self.is_cs_plus,
            my_char: self.my_char,
            booster: self.booster,
            caret: self.caret.clone(),
            world: self.world.clone(),
            tex_sizes: self.tex_sizes.clone(),
            textscript: self.textscript.clone(),
        }
    }
}

impl EngineConstants {
    pub fn defaults() -> Self {
        EngineConstants {
            is_cs_plus: false,
            my_char: MyCharConsts {
                cond: 0x80,
                flags: 0,
                equip: 0,
                direction: Direction::Right,
                view: Rect { left: 8 * 0x200, top: 8 * 0x200, right: 8 * 0x200, bottom: 8 * 0x200 },
                hit: Rect { left: 5 * 0x200, top: 8 * 0x200, right: 5 * 0x200, bottom: 8 * 0x200 },
                life: 3,
                max_life: 3,
                unit: 0,
                air_physics: PhysicsConsts {
                    max_dash: 0x32c,
                    max_move: 0x5ff,
                    gravity_air: 0x20,
                    gravity_ground: 0x50,
                    dash_air: 0x20,
                    dash_ground: 0x55,
                    resist: 0x33,
                    jump: 0x500,
                },
                water_physics: PhysicsConsts {
                    max_dash: 0x196,
                    max_move: 0x2ff,
                    gravity_air: 0x10,
                    gravity_ground: 0x28,
                    dash_air: 0x10,
                    dash_ground: 0x2a,
                    resist: 0x19,
                    jump: 0x280,
                },
                animations_left: [
                    Rect { left: 0, top: 0, right: 16, bottom: 16 },
                    Rect { left: 16, top: 0, right: 32, bottom: 16 },
                    Rect { left: 0, top: 0, right: 16, bottom: 16 },
                    Rect { left: 32, top: 0, right: 48, bottom: 16 },
                    Rect { left: 0, top: 0, right: 16, bottom: 16 },
                    Rect { left: 48, top: 0, right: 64, bottom: 16 },
                    Rect { left: 64, top: 0, right: 80, bottom: 16 },
                    Rect { left: 48, top: 0, right: 64, bottom: 16 },
                    Rect { left: 80, top: 0, right: 96, bottom: 16 },
                    Rect { left: 48, top: 0, right: 64, bottom: 16 },
                    Rect { left: 96, top: 0, right: 112, bottom: 16 },
                    Rect { left: 112, top: 0, right: 128, bottom: 16 },
                ],
                animations_right: [
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 16, top: 16, right: 32, bottom: 32 },
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 32, top: 16, right: 48, bottom: 32 },
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 48, top: 16, right: 64, bottom: 32 },
                    Rect { left: 64, top: 16, right: 80, bottom: 32 },
                    Rect { left: 48, top: 16, right: 64, bottom: 32 },
                    Rect { left: 80, top: 16, right: 96, bottom: 32 },
                    Rect { left: 48, top: 16, right: 64, bottom: 32 },
                    Rect { left: 96, top: 16, right: 112, bottom: 32 },
                    Rect { left: 112, top: 16, right: 128, bottom: 32 },
                ],
            },
            booster: BoosterConsts {
                fuel: 50,
                b2_0_up: -0x5ff,
                b2_0_up_nokey: -0x5ff,
                b2_0_down: 0x5ff,
                b2_0_left: -0x5ff,
                b2_0_right: 0x5ff,
            },
            caret: CaretConsts {
                offsets: [
                    (0, 0),
                    (4 * 0x200, 4 * 0x200),
                    (8 * 0x200, 8 * 0x200),
                    (8 * 0x200, 8 * 0x200),
                    (8 * 0x200, 8 * 0x200),
                    (4 * 0x200, 4 * 0x200),
                    (8 * 0x200, 8 * 0x200),
                    (4 * 0x200, 4 * 0x200),
                    (8 * 0x200, 8 * 0x200),
                    (8 * 0x200, 8 * 0x200),
                    (28 * 0x200, 8 * 0x200),
                    (4 * 0x200, 4 * 0x200),
                    (16 * 0x200, 16 * 0x200),
                    (4 * 0x200, 4 * 0x200),
                    (20 * 0x200, 20 * 0x200),
                    (4 * 0x200, 4 * 0x200),
                    (20 * 0x200, 4 * 0x200),
                    (52 * 0x200, 4 * 0x200),
                ],
                bubble_left_rects: vec![
                    Rect { left: 0, top: 64, right: 8, bottom: 72 },
                    Rect { left: 8, top: 64, right: 16, bottom: 72 },
                    Rect { left: 16, top: 64, right: 24, bottom: 72 },
                    Rect { left: 24, top: 64, right: 32, bottom: 72 },
                ],
                bubble_right_rects: vec![
                    Rect { left: 64, top: 24, right: 72, bottom: 32 },
                    Rect { left: 72, top: 24, right: 80, bottom: 32 },
                    Rect { left: 80, top: 24, right: 88, bottom: 32 },
                    Rect { left: 88, top: 24, right: 96, bottom: 32 },
                ],
                little_particles_rects: vec![
                    Rect { left: 56, top: 24, right: 64, bottom: 32 },
                    Rect { left: 0, top: 0, right: 0, bottom: 0 },
                ],
                exhaust_rects: vec![
                    Rect { left: 56, top: 0, right: 64, bottom: 8 },
                    Rect { left: 64, top: 0, right: 72, bottom: 8 },
                    Rect { left: 72, top: 0, right: 80, bottom: 8 },
                    Rect { left: 80, top: 0, right: 88, bottom: 8 },
                    Rect { left: 88, top: 0, right: 96, bottom: 8 },
                    Rect { left: 96, top: 0, right: 104, bottom: 8 },
                    Rect { left: 104, top: 0, right: 112, bottom: 8 },
                ],
                question_left_rect: Rect { left: 0, top: 80, right: 16, bottom: 96 },
                question_right_rect: Rect { left: 48, top: 64, right: 64, bottom: 80 },
            },
            world: WorldConsts {
                snack_rect: Rect { left: 256, top: 48, right: 272, bottom: 64 },
            },
            tex_sizes: hashmap! {
                str!("ArmsImage") => (256, 16),
                str!("Arms") => (320, 200),
                str!("bk0") => (64, 64),
                str!("bkBlack") => (64, 64),
                str!("bkBlue") => (64, 64),
                str!("bkFall") => (64, 64),
                str!("bkFog") => (320, 240),
                str!("bkFog480fix") => (480, 272), // nxengine
                str!("bkGard") => (48, 64),
                str!("bkGray") => (64, 64),
                str!("bkGreen") => (64, 64),
                str!("bkHellish") => (320, 240), // nxengine
                str!("bkHellish480fix") => (480, 272), // nxengine
                str!("bkLight") => (320, 240), // nxengine
                str!("bkLight480fix") => (480, 272), // nxengine
                str!("bkMaze") => (64, 64),
                str!("bkMoon") => (320, 240),
                str!("bkMoon480fix") => (480, 272), // nxengine
                str!("bkRed") => (32, 32),
                str!("bkSunset") => (320, 240), // nxengine
                str!("bkSunset480fix") => (480, 272), // nxengine
                str!("bkWater") => (32, 48),
                str!("Bullet") => (320, 176),
                str!("Caret") => (320, 240),
                str!("casts") => (320, 240),
                str!("Face") => (288, 240),
                str!("Face_0") => (288, 240), // nxengine
                str!("Face_1") => (288, 240), // nxengine
                str!("Face_2") => (288, 240), // nxengine
                str!("Fade") => (256, 32),
                str!("ItemImage") => (256, 128),
                str!("Loading") => (64, 8),
                str!("MyChar") => (200, 64),
                str!("Npc/Npc0") => (32, 32),
                str!("Npc/NpcAlmo1") => (320, 240),
                str!("Npc/NpcAlmo2") => (320, 240),
                str!("Npc/NpcBallos") => (320, 240),
                str!("Npc/NpcBllg") => (320, 96),
                str!("Npc/NpcCemet") => (320, 112),
                str!("Npc/NpcCent") => (320, 192),
                str!("Npc/NpcCurly") => (256, 80),
                str!("Npc/NpcDark") => (160, 64),
                str!("Npc/NpcDr") => (320, 240),
                str!("Npc/NpcEggs1") => (320, 112),
                str!("Npc/NpcEggs2") => (320, 128),
                str!("Npc/NpcFrog") => (320, 240),
                str!("Npc/NpcGuest") => (320, 184),
                str!("Npc/NpcHell") => (320, 160),
                str!("Npc/NpcHeri") => (320, 128),
                str!("Npc/NpcIronH") => (320, 72),
                str!("Npc/NpcIsland") => (320, 80),
                str!("Npc/NpcKings") => (96, 48),
                str!("Npc/NpcMaze") => (320, 192),
                str!("Npc/NpcMiza") => (320, 240),
                str!("Npc/NpcMoon") => (320, 128),
                str!("Npc/NpcOmg") => (320, 120),
                str!("Npc/NpcPlant") => (320, 48),
                str!("Npc/NpcPress") => (320, 240),
                str!("Npc/NpcPriest") => (320, 240),
                str!("Npc/NpcRavil") => (320, 168),
                str!("Npc/NpcRed") => (320, 144),
                str!("Npc/NpcRegu") => (320, 240),
                str!("Npc/NpcSand") => (320, 176),
                str!("Npc/NpcStream") => (64, 32),
                str!("Npc/NpcSym") => (320, 240),
                str!("Npc/NpcToro") => (320, 144),
                str!("Npc/NpcTwinD") => (320, 144),
                str!("Npc/NpcWeed") => (320, 240),
                str!("Npc/NpcX") => (320, 240),
                str!("Resource/BITMAP/Credit01") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit02") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit03") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit04") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit05") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit06") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit07") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit08") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit09") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit10") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit11") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit12") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit14") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit15") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit16") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit17") => (160, 240), // cse2
                str!("Resource/BITMAP/Credit18") => (160, 240), // cse2
                str!("Resource/BITMAP/pixel") => (160, 16), // cse2
                str!("Resource/CURSOR/CURSOR_IKA") => (32, 32), // cse2
                str!("Resource/CURSOR/CURSOR_NORMAL") => (32, 32), // cse2
                str!("StageImage") => (256, 16),
                str!("Stage/Prt0") => (32, 32),
                str!("Stage/PrtAlmond") => (256, 96),
                str!("Stage/PrtBarr") => (256, 88),
                str!("Stage/PrtCave") => (256, 80),
                str!("Stage/PrtCent") => (256, 128),
                str!("Stage/PrtEggIn") => (256, 80),
                str!("Stage/PrtEggs") => (256, 240),
                str!("Stage/PrtEggX") => (256, 240),
                str!("Stage/PrtFall") => (256, 128),
                str!("Stage/PrtGard") => (256, 97),
                str!("Stage/PrtHell") => (256, 240),
                str!("Stage/PrtJail") => (256, 128),
                str!("Stage/PrtLabo") => (128, 64),
                str!("Stage/PrtMaze") => (256, 160),
                str!("Stage/PrtMimi") => (256, 160),
                str!("Stage/PrtOside") => (256, 64),
                str!("Stage/PrtPens") => (256, 64),
                str!("Stage/PrtRiver") => (256, 96),
                str!("Stage/PrtSand") => (256, 112),
                str!("Stage/PrtStore") => (256, 112),
                str!("Stage/PrtWeed") => (256, 128),
                str!("Stage/PrtWhite") => (256, 240),
                str!("TextBox") => (244, 144),
                str!("Title") => (320, 48),
            },
            textscript: TextScriptConsts {
                encoding: TextScriptEncoding::UTF8,
                textbox_rect_top: Rect { left: 0, top: 0, right: 244, bottom: 8 },
                textbox_rect_middle: Rect { left: 0, top: 8, right: 244, bottom: 16 },
                textbox_rect_bottom: Rect { left: 0, top: 16, right: 244, bottom: 24 },
            },
        }
    }

    pub fn apply_csplus_patches(&mut self) {
        info!("Applying Cave Story+ constants patches...");

        self.is_cs_plus = true;
        self.tex_sizes.insert(str!("Caret"), (320, 320));
        self.tex_sizes.insert(str!("MyChar"), (200, 384));
    }
}
