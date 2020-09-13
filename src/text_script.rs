use std::collections::HashMap;
use std::io;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::iter::Peekable;
use std::ops::Not;
use std::str::FromStr;

use byteorder::ReadBytesExt;
use itertools::Itertools;
use num_derive::FromPrimitive;
use num_traits::{clamp, FromPrimitive};

use crate::{SharedGameState, str};
use crate::bitfield;
use crate::common::{Direction, FadeDirection, FadeState};
use crate::encoding::{read_cur_shift_jis, read_cur_wtf8};
use crate::entity::GameEntity;
use crate::ggez::{Context, GameResult};
use crate::ggez::GameError::ParseError;
use crate::player::ControlMode;
use crate::scene::game_scene::GameScene;
use crate::weapon::WeaponType;

/// Engine's text script VM operation codes.
#[derive(EnumString, Debug, FromPrimitive, PartialEq)]
#[repr(i32)]
pub enum OpCode {
    // ---- Internal opcodes (used by bytecode, no TSC representation)
    /// internal: no operation
    _NOP = 0,
    /// internal: unimplemented
    _UNI,
    /// internal: string marker
    _STR,
    /// internal: implicit END marker
    _END,

    // ---- Official opcodes ----
    /// <BOAxxxx, start boss animation
    BOA,
    /// <BSLxxxx, start boss fight
    BSL,

    /// <FOBxxxx, Focus on boss
    FOB,
    FOM,
    FON,
    FLA,
    QUA,

    UNI,
    HMC,
    SMC,
    MM0,
    MOV,
    MYB,
    MYD,
    TRA,

    END,
    FRE,
    FAI,
    FAO,
    WAI,
    WAS,
    KEY,
    PRI,
    NOD,
    CAT,
    SAT,
    TUR,
    CLO,
    CLR,
    FAC,
    GIT,
    MS2,
    MS3,
    MSG,
    NUM,

    ANP,
    CNP,
    INP,
    MNP,
    DNA,
    DNP,
    SNP,

    #[strum(serialize = "FL-")]
    FLm,
    #[strum(serialize = "FL+")]
    FLp,
    #[strum(serialize = "MP+")]
    MPp,
    #[strum(serialize = "SK-")]
    SKm,
    #[strum(serialize = "SK+")]
    SKp,

    #[strum(serialize = "EQ+")]
    EQp,
    #[strum(serialize = "EQ-")]
    EQm,
    #[strum(serialize = "ML+")]
    MLp,
    #[strum(serialize = "IT+")]
    ITp,
    #[strum(serialize = "IT-")]
    ITm,
    #[strum(serialize = "AM+")]
    AMp,
    #[strum(serialize = "AM-")]
    AMm,
    TAM,

    UNJ,
    NCJ,
    ECJ,
    FLJ,
    ITJ,
    MPJ,
    YNJ,
    SKJ,
    EVE,
    AMJ,

    MLP,
    MNA,
    CMP,
    SMP,

    CRE,
    XX1,
    CIL,
    SIL,
    ESC,
    INI,
    LDP,
    #[strum(serialize = "PS+")]
    PSp,
    SLP,
    ZAM,

    #[strum(serialize = "AE+")]
    AEp,
    #[strum(serialize = "LI+")]
    LIp,

    SVP,
    STC,

    SOU,
    CMU,
    FMU,
    RMU,
    CPS,
    SPS,
    CSS,
    SSS,

    // ---- Cave Story+ specific opcodes ----
    /// <ACHXXXX, triggers a Steam achievement.
    ACH,

    // ---- Custom opcodes, for use by modders ----
}

bitfield! {
  pub struct TextScriptFlags(u16);
  impl Debug;
  pub render, set_render: 0;
  pub background_visible, set_background_visible: 1;
  pub flag_x10, set_flag_x10: 4;
  pub position_top, set_position_top: 5;
  pub flag_x40, set_flag_x40: 6;
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum TextScriptEncoding {
    UTF8 = 0,
    ShiftJIS,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum TextScriptLine {
    Line1 = 0,
    Line2,
    Line3,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum ConfirmSelection {
    Yes,
    No,
}

impl Not for ConfirmSelection {
    type Output = ConfirmSelection;

    fn not(self) -> ConfirmSelection {
        if self == ConfirmSelection::Yes {
            ConfirmSelection::No
        } else {
            ConfirmSelection::Yes
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum TextScriptExecutionState {
    Ended,
    Running(u16, u32),
    Msg(u16, u32, u32, u8),
    WaitTicks(u16, u32, u16),
    WaitInput(u16, u32),
    WaitStanding(u16, u32),
    WaitConfirmation(u16, u32, u16, u8, ConfirmSelection),
    WaitFade(u16, u32),
}

pub struct TextScriptVM {
    pub scripts: TextScriptVMScripts,
    pub state: TextScriptExecutionState,
    pub flags: TextScriptFlags,
    /// Toggle for non-strict TSC parsing because English versions of CS+ (both AG and Nicalis release)
    /// modified the events carelessly and since original Pixel's engine hasn't enforced constraints
    /// while parsing no one noticed them.
    pub strict_mode: bool,
    pub suspend: bool,
    pub face: u16,
    pub item: u16,
    pub current_line: TextScriptLine,
    pub line_1: Vec<char>,
    pub line_2: Vec<char>,
    pub line_3: Vec<char>,
}

impl Default for TextScriptVM {
    fn default() -> Self {
        TextScriptVM::new()
    }
}

pub struct TextScriptVMScripts {
    pub global_script: TextScript,
    pub scene_script: TextScript,
}

impl TextScriptVMScripts {
    pub fn find_script(&self, event_num: u16) -> Option<&Vec<u8>> {
        if let Some(tsc) = self.scene_script.event_map.get(&event_num) {
            return Some(tsc);
        } else if let Some(tsc) = self.global_script.event_map.get(&event_num) {
            return Some(tsc);
        }

        None
    }
}

fn read_cur_varint(cursor: &mut Cursor<&Vec<u8>>) -> GameResult<i32> {
    let mut result = 0u32;

    for o in 0..5 {
        let n = cursor.read_u8()?;
        result |= (n as u32 & 0x7f) << (o * 7);

        if n & 0x80 == 0 {
            break;
        }
    }

    Ok(((result << 31) ^ (result >> 1)) as i32)
}

impl TextScriptVM {
    pub fn new() -> Self {
        Self {
            scripts: TextScriptVMScripts {
                global_script: TextScript::new(),
                scene_script: TextScript::new(),
            },
            state: TextScriptExecutionState::Ended,
            strict_mode: false,
            suspend: true,
            flags: TextScriptFlags(0),
            item: 0,
            face: 0,
            current_line: TextScriptLine::Line1,
            line_1: Vec::with_capacity(24),
            line_2: Vec::with_capacity(24),
            line_3: Vec::with_capacity(24),
        }
    }

    pub fn set_global_script(&mut self, script: TextScript) {
        self.scripts.global_script = script;
        if !self.suspend { self.reset(); }
    }

    pub fn set_scene_script(&mut self, script: TextScript) {
        self.scripts.scene_script = script;
        if !self.suspend { self.reset(); }
    }

    pub fn append_global_script(&mut self, script: TextScript) {
        for (key, val) in script.event_map {
            self.scripts.global_script.event_map.insert(key, val);
        }

        if !self.suspend { self.reset(); }
    }

    pub fn append_scene_script(&mut self, script: TextScript) {
        for (key, val) in script.event_map {
            self.scripts.scene_script.event_map.insert(key, val);
        }

        if !self.suspend { self.reset(); }
    }

    pub fn reset(&mut self) {
        self.state = TextScriptExecutionState::Ended;
        self.clear_text_box();
    }

    pub fn clear_text_box(&mut self) {
        self.flags.0 = 0;
        self.face = 0;
        self.item = 0;
        self.current_line = TextScriptLine::Line1;
        self.line_1.clear();
        self.line_2.clear();
        self.line_3.clear();
    }

    pub fn start_script(&mut self, event_num: u16) {
        self.reset();
        self.state = TextScriptExecutionState::Running(event_num, 0);

        log::info!("Started script: #{:04}", event_num);
    }

    pub fn run(state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context) -> GameResult {
        loop {
            if state.textscript_vm.suspend { break; }

            match state.textscript_vm.state {
                TextScriptExecutionState::Ended => {
                    break;
                }
                TextScriptExecutionState::Running(event, ip) => {
                    state.control_flags.set_flag_x01(true);
                    state.control_flags.set_interactions_disabled(true);
                    state.textscript_vm.state = TextScriptVM::execute(event, ip, state, game_scene, ctx)?;

                    if state.textscript_vm.state == TextScriptExecutionState::Ended {
                        state.textscript_vm.reset();
                    }
                }
                TextScriptExecutionState::Msg(event, ip, remaining, counter) => {
                    if counter > 0 {
                        state.textscript_vm.state = TextScriptExecutionState::Msg(event, ip, remaining, counter - 1);
                        break;
                    }

                    if let Some(bytecode) = state.textscript_vm.scripts.find_script(event) {
                        let mut cursor = Cursor::new(bytecode);
                        cursor.seek(SeekFrom::Start(ip as u64))?;

                        let chr = std::char::from_u32(read_cur_varint(&mut cursor)? as u32).unwrap_or('\u{fffd}');

                        match chr {
                            '\n' if state.textscript_vm.current_line == TextScriptLine::Line1 => {
                                state.textscript_vm.current_line = TextScriptLine::Line2;
                            }
                            '\n' if state.textscript_vm.current_line == TextScriptLine::Line2 => {
                                state.textscript_vm.current_line = TextScriptLine::Line3;
                            }
                            '\n' => {
                                state.textscript_vm.line_1.clear();
                                state.textscript_vm.line_1.append(&mut state.textscript_vm.line_2);
                                state.textscript_vm.line_2.append(&mut state.textscript_vm.line_3);
                            }
                            '\r' => {}
                            _ if state.textscript_vm.current_line == TextScriptLine::Line1 => {
                                state.textscript_vm.line_1.push(chr);
                            }
                            _ if state.textscript_vm.current_line == TextScriptLine::Line2 => {
                                state.textscript_vm.line_2.push(chr);
                            }
                            _ if state.textscript_vm.current_line == TextScriptLine::Line3 => {
                                state.textscript_vm.line_3.push(chr);
                            }
                            _ => {}
                        }

                        if remaining > 1 {
                            let ticks = if state.key_state.jump() || state.key_state.fire() { 1 } else { 4 };
                            state.textscript_vm.state = TextScriptExecutionState::Msg(event, cursor.position() as u32, remaining - 1, ticks);
                        } else {
                            state.textscript_vm.state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    } else {
                        state.textscript_vm.reset();
                    }
                }
                TextScriptExecutionState::WaitTicks(event, ip, ticks) => {
                    if ticks == 0 {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    } else {
                        state.textscript_vm.state = TextScriptExecutionState::WaitTicks(event, ip, ticks - 1);
                        break;
                    }
                }
                TextScriptExecutionState::WaitConfirmation(event, ip, no_event, wait, selection) => {
                    if wait > 0 {
                        state.textscript_vm.state = TextScriptExecutionState::WaitConfirmation(event, ip, no_event, wait - 1, selection);
                        break;
                    }

                    if state.key_trigger.left() || state.key_trigger.right() {
                        state.textscript_vm.state = TextScriptExecutionState::WaitConfirmation(event, ip, no_event, 0, !selection);
                        break;
                    }

                    if state.key_trigger.jump() {
                        match selection {
                            ConfirmSelection::Yes => {
                                state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                            }
                            ConfirmSelection::No => {
                                state.textscript_vm.state = TextScriptExecutionState::Running(no_event, 0);
                            }
                        }
                    }

                    break;
                }
                TextScriptExecutionState::WaitStanding(event, ip) => {
                    if game_scene.player.flags.hit_bottom_wall() {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    }
                    break;
                }
                TextScriptExecutionState::WaitInput(event, ip) => {
                    if state.key_trigger.jump() || state.key_trigger.fire() {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    }
                    break;
                }
                TextScriptExecutionState::WaitFade(event, ip) => {
                    if state.fade_state == FadeState::Hidden || state.fade_state == FadeState::Visible {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn execute(event: u16, ip: u32, state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context) -> GameResult<TextScriptExecutionState> {
        let mut exec_state = state.textscript_vm.state;
        let mut tick_npc = 0u16;

        if let Some(bytecode) = state.textscript_vm.scripts.find_script(event) {
            let mut cursor = Cursor::new(bytecode);
            cursor.seek(SeekFrom::Start(ip as u64))?;

            let op_maybe: Option<OpCode> = FromPrimitive::from_i32(read_cur_varint(&mut cursor)
                .unwrap_or_else(|_| OpCode::END as i32));

            if let Some(op) = op_maybe {
                println!("opcode: {:?}", op);
                match op {
                    OpCode::_NOP => {
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::_UNI => {}
                    OpCode::_STR => {
                        let mut len = read_cur_varint(&mut cursor)? as u32;
                        if state.textscript_vm.flags.render() {
                            exec_state = TextScriptExecutionState::Msg(event, cursor.position() as u32, len, 4);
                        } else {
                            while len > 0 {
                                len -= 1;
                                let _ = read_cur_varint(&mut cursor)?;
                            }
                            // simply skip the text if we aren't in message mode.
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::_END | OpCode::END => {
                        state.control_flags.set_flag_x01(true);
                        state.control_flags.set_control_enabled(true);
                        state.control_flags.set_interactions_disabled(false);

                        state.textscript_vm.flags.set_render(false);
                        state.textscript_vm.flags.set_background_visible(false);

                        game_scene.player.update_target = true;

                        exec_state = TextScriptExecutionState::Ended;
                    }
                    OpCode::PRI => {
                        state.control_flags.set_flag_x01(false);
                        state.control_flags.set_control_enabled(false);

                        game_scene.player.shock_counter = 0;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::KEY => {
                        state.control_flags.set_flag_x01(true);
                        state.control_flags.set_control_enabled(false);

                        game_scene.player.up = false;
                        game_scene.player.down = false;
                        game_scene.player.shock_counter = 0;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FRE => {
                        state.control_flags.set_flag_x01(true);
                        state.control_flags.set_control_enabled(true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MYD => {
                        let new_direction = read_cur_varint(&mut cursor)? as usize;
                        if let Some(direction) = Direction::from_int(new_direction) {
                            game_scene.player.direction = direction;
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MYB => {
                        let new_direction = read_cur_varint(&mut cursor)? as usize;

                        game_scene.player.vel_y = -0x200;

                        if let Some(direction) = Direction::from_int(new_direction) {
                            match direction {
                                Direction::Left => { game_scene.player.vel_x = 0x200 }
                                Direction::Up => { game_scene.player.vel_y = 0x200 }
                                Direction::Right => { game_scene.player.vel_x = -0x200 }
                                Direction::Bottom => { game_scene.player.vel_y = -0x200 }
                            }
                        } else {
                            // todo npc direction dependent bump
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SMC => {
                        game_scene.player.cond.set_hidden(false);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::HMC => {
                        game_scene.player.cond.set_hidden(true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::WAI => {
                        let ticks = read_cur_varint(&mut cursor)? as u16;

                        exec_state = TextScriptExecutionState::WaitTicks(event, cursor.position() as u32, ticks);
                    }
                    OpCode::WAS => {
                        exec_state = TextScriptExecutionState::WaitStanding(event, cursor.position() as u32);
                    }
                    OpCode::NOD => {
                        exec_state = TextScriptExecutionState::WaitInput(event, cursor.position() as u32);
                    }
                    OpCode::FLp | OpCode::FLm => {
                        let flag_num = read_cur_varint(&mut cursor)? as usize;
                        state.game_flags.set(flag_num, op == OpCode::FLp);
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FLJ => {
                        let flag_num = read_cur_varint(&mut cursor)? as usize;
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        if let Some(true) = state.game_flags.get(flag_num) {
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::ITJ => {
                        let item_id = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if game_scene.inventory.has_item(item_id) {
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::AMJ => {
                        let weapon = read_cur_varint(&mut cursor)? as u8;
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon);

                        if weapon_type.is_some() && game_scene.inventory.has_weapon(weapon_type.unwrap()) {
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::NCJ => {
                        let npc_id = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if game_scene.npc_map.is_alive(npc_id) {
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::ECJ => {
                        let npc_event_num = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if game_scene.npc_map.is_alive_by_event(npc_event_num) {
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::EVE => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        exec_state = TextScriptExecutionState::Running(event_num, 0);
                    }
                    OpCode::MM0 => {
                        game_scene.player.vel_x = 0;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CMP => {
                        let pos_x = read_cur_varint(&mut cursor)? as usize;
                        let pos_y = read_cur_varint(&mut cursor)? as usize;
                        let tile_type = read_cur_varint(&mut cursor)? as u8;

                        if let Some(ptr) = game_scene.stage.map.tiles.get_mut(pos_y * game_scene.stage.map.width + pos_x) {
                            *ptr = tile_type;
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MLp => {
                        let life = read_cur_varint(&mut cursor)? as usize;
                        game_scene.player.life += life;
                        game_scene.player.max_life += life;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FAC => {
                        let face = read_cur_varint(&mut cursor)? as u16;
                        state.textscript_vm.face = face;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CLR => {
                        state.textscript_vm.current_line = TextScriptLine::Line1;
                        state.textscript_vm.line_1.clear();
                        state.textscript_vm.line_2.clear();
                        state.textscript_vm.line_3.clear();

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MSG | OpCode::MS2 | OpCode::MS3 => {
                        state.textscript_vm.current_line = TextScriptLine::Line1;
                        state.textscript_vm.line_1.clear();
                        state.textscript_vm.line_2.clear();
                        state.textscript_vm.line_3.clear();
                        state.textscript_vm.flags.set_render(true);
                        state.textscript_vm.flags.set_background_visible(op != OpCode::MS2);
                        state.textscript_vm.flags.set_flag_x10(state.textscript_vm.flags.flag_x40());
                        state.textscript_vm.flags.set_position_top(op != OpCode::MSG);
                        if op == OpCode::MS2 {
                            state.textscript_vm.face = 0;
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CLO => {
                        state.textscript_vm.flags.set_render(false);
                        state.textscript_vm.flags.set_background_visible(false);
                        state.textscript_vm.flags.set_flag_x10(false);
                        state.textscript_vm.flags.set_position_top(false);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::YNJ => {
                        let event_no = read_cur_varint(&mut cursor)? as u16;

                        exec_state = TextScriptExecutionState::WaitConfirmation(event, cursor.position() as u32, event_no, 16, ConfirmSelection::Yes);
                    }
                    OpCode::GIT => {
                        let item = read_cur_varint(&mut cursor)? as u16;
                        state.textscript_vm.item = item;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::TRA => {
                        let map_id = read_cur_varint(&mut cursor)? as usize;
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        let pos_x = read_cur_varint(&mut cursor)? as isize * 16 * 0x200;
                        let pos_y = read_cur_varint(&mut cursor)? as isize * 16 * 0x200;

                        let mut new_scene = GameScene::new(state, ctx, map_id)?;
                        new_scene.inventory = game_scene.inventory.clone();
                        new_scene.player = game_scene.player.clone();
                        new_scene.player.vel_x = 0;
                        new_scene.player.vel_y = 0;
                        new_scene.player.x = pos_x;
                        new_scene.player.y = pos_y;

                        state.textscript_vm.flags.0 = 0;
                        state.textscript_vm.face = 0;
                        state.textscript_vm.item = 0;
                        state.textscript_vm.current_line = TextScriptLine::Line1;
                        state.textscript_vm.line_1.clear();
                        state.textscript_vm.line_2.clear();
                        state.textscript_vm.line_3.clear();
                        state.textscript_vm.suspend = true;
                        state.next_scene = Some(Box::new(new_scene));

                        log::info!("Transitioning to stage {}, with script #{:04}", map_id, event_num);
                        exec_state = TextScriptExecutionState::Running(event_num, 0);
                    }
                    OpCode::MOV => {
                        let pos_x = read_cur_varint(&mut cursor)? as isize * 16 * 0x200;
                        let pos_y = read_cur_varint(&mut cursor)? as isize * 16 * 0x200;

                        game_scene.player.vel_x = 0;
                        game_scene.player.vel_y = 0;
                        game_scene.player.x = pos_x;
                        game_scene.player.y = pos_y;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::UNI => {
                        let control_mode = read_cur_varint(&mut cursor)? as u8;

                        let mode: Option<ControlMode> = FromPrimitive::from_u8(control_mode);
                        if let Some(mode) = mode {
                            game_scene.player.control_mode = mode;
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FAI => {
                        let fade_type = read_cur_varint(&mut cursor)? as usize;

                        if let Some(direction) = FadeDirection::from_int(fade_type) {
                            state.fade_state = FadeState::FadeIn(15, direction);
                        }

                        exec_state = TextScriptExecutionState::WaitFade(event, cursor.position() as u32);
                    }
                    OpCode::FAO => {
                        let fade_type = read_cur_varint(&mut cursor)? as usize;

                        if let Some(direction) = FadeDirection::from_int(fade_type) {
                            state.fade_state = FadeState::FadeOut(-15, direction.opposite());
                        }

                        exec_state = TextScriptExecutionState::WaitFade(event, cursor.position() as u32);
                    }
                    OpCode::QUA => {
                        let count = read_cur_varint(&mut cursor)? as u16;

                        state.quake_counter = count;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MNA => {
                        game_scene.display_map_name(160);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CMU => {
                        let song_id = read_cur_varint(&mut cursor)? as usize;
                        state.sound_manager.play_song(song_id, &state.constants, ctx)?;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FMU => {
                        state.sound_manager.play_song(0, &state.constants, ctx)?;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::RMU => {
                        state.sound_manager.restore_state()?;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::DNP => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        game_scene.npc_map.remove_by_event(event_num, &mut state.game_flags);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FOM => {
                        let ticks = read_cur_varint(&mut cursor)? as isize;
                        game_scene.frame.wait = ticks;
                        game_scene.player.update_target = true;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FON => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        let ticks = read_cur_varint(&mut cursor)? as isize;
                        game_scene.frame.wait = ticks;
                        game_scene.player.update_target = false;

                        for npc_id in game_scene.npc_map.npc_ids.iter() {
                            if let Some(npc_cell) = game_scene.npc_map.npcs.get(npc_id) {
                                let npc = npc_cell.borrow();

                                if event_num == npc.event_num {
                                    game_scene.player.target_x = npc.x;
                                    game_scene.player.target_y = npc.y;
                                    break;
                                }
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::ANP => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        let action_num = read_cur_varint(&mut cursor)? as u16;
                        let direction = read_cur_varint(&mut cursor)? as usize;

                        for npc_id in game_scene.npc_map.npc_ids.iter() {
                            if let Some(npc_cell) = game_scene.npc_map.npcs.get(npc_id) {
                                let mut npc = npc_cell.borrow_mut();

                                if npc.cond.alive() && npc.event_num == event_num {
                                    npc.action_num = action_num;

                                    if direction == 4 {
                                        npc.direction = if game_scene.player.x < npc.x {
                                            Direction::Right
                                        } else {
                                            Direction::Left
                                        };
                                    } else if let Some(dir) = Direction::from_int(direction) {
                                        npc.direction = dir;
                                    }

                                    break;
                                }
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CNP | OpCode::INP => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        let new_type = read_cur_varint(&mut cursor)? as u16;
                        let direction = read_cur_varint(&mut cursor)? as usize;

                        for npc_id in game_scene.npc_map.npc_ids.iter() {
                            if let Some(npc_cell) = game_scene.npc_map.npcs.get(npc_id) {
                                let mut npc = npc_cell.borrow_mut();

                                if npc.cond.alive() && npc.event_num == event_num {
                                    npc.npc_flags.set_solid_soft(false);
                                    npc.npc_flags.set_ignore_tile_44(false);
                                    npc.npc_flags.set_invulnerable(false);
                                    npc.npc_flags.set_ignore_solidity(false);
                                    npc.npc_flags.set_bouncy(false);
                                    npc.npc_flags.set_shootable(false);
                                    npc.npc_flags.set_solid_hard(false);
                                    npc.npc_flags.set_rear_and_top_not_hurt(false);
                                    npc.npc_flags.set_show_damage(false);

                                    if op == OpCode::INP {
                                        npc.npc_flags.set_event_when_touched(true);
                                    }

                                    npc.npc_type = new_type;
                                    npc.display_bounds = state.npc_table.get_display_bounds(new_type);
                                    npc.hit_bounds = state.npc_table.get_hit_bounds(new_type);
                                    let entry = state.npc_table.get_entry(new_type).unwrap().to_owned();
                                    npc.npc_flags.0 |= entry.npc_flags.0;
                                    npc.life = entry.life;

                                    npc.cond.set_alive(true);
                                    npc.action_num = 0;
                                    npc.action_counter = 0;
                                    npc.anim_num = 0;
                                    npc.anim_counter = 0;
                                    npc.vel_x = 0;
                                    npc.vel_y = 0;

                                    if direction == 4 {
                                        npc.direction = if game_scene.player.x < npc.x {
                                            Direction::Right
                                        } else {
                                            Direction::Left
                                        };
                                    } else if let Some(dir) = Direction::from_int(direction) {
                                        npc.direction = dir;
                                    }

                                    tick_npc = *npc_id;

                                    break;
                                }
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MNP => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        let x = read_cur_varint(&mut cursor)? as isize;
                        let y = read_cur_varint(&mut cursor)? as isize;
                        let direction = read_cur_varint(&mut cursor)? as usize;

                        for npc_id in game_scene.npc_map.npc_ids.iter() {
                            if let Some(npc_cell) = game_scene.npc_map.npcs.get(npc_id) {
                                let mut npc = npc_cell.borrow_mut();

                                if npc.cond.alive() && npc.event_num == event_num {
                                    npc.x = x * 16 * 0x200;
                                    npc.y = y * 16 * 0x200;

                                    if direction == 4 {
                                        npc.direction = if game_scene.player.x < npc.x {
                                            Direction::Right
                                        } else {
                                            Direction::Left
                                        };
                                    } else if let Some(dir) = Direction::from_int(direction) {
                                        npc.direction = dir;
                                    }

                                    break;
                                }
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::LIp => {
                        let life = read_cur_varint(&mut cursor)? as usize;

                        game_scene.player.life = clamp(game_scene.player.life + life, 0, game_scene.player.max_life);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::ITp => {
                        let item_id = read_cur_varint(&mut cursor)? as u16;

                        game_scene.inventory.add_item(item_id);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::ITm => {
                        let item_id = read_cur_varint(&mut cursor)? as u16;

                        game_scene.inventory.remove_item(item_id);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::AMp => {
                        let weapon_id = read_cur_varint(&mut cursor)? as u8;
                        let max_ammo = read_cur_varint(&mut cursor)? as u16;
                        let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon_id);

                        if let Some(wtype) = weapon_type {
                            game_scene.inventory.add_weapon(wtype, max_ammo);
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::AMm => {
                        let weapon_id = read_cur_varint(&mut cursor)? as u8;
                        let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon_id);

                        if let Some(wtype) = weapon_type {
                            game_scene.inventory.remove_weapon(wtype);
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::AEp => {
                        game_scene.inventory.refill_all_ammo();

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::ZAM => {
                        game_scene.inventory.reset_all_weapon_xp();

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::EQp => {
                        let mask = read_cur_varint(&mut cursor)? as u16;

                        game_scene.player.equip.0 |= mask;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::EQm => {
                        let mask = read_cur_varint(&mut cursor)? as u16;

                        game_scene.player.equip.0 &= !mask;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // unimplemented opcodes
                    // Zero operands
                    OpCode::CAT | OpCode::CIL | OpCode::CPS |
                    OpCode::CRE | OpCode::CSS | OpCode::ESC | OpCode::FLA |
                    OpCode::INI | OpCode::LDP | OpCode::MLP |
                    OpCode::SAT | OpCode::SLP | OpCode::SPS |
                    OpCode::STC | OpCode::SVP | OpCode::TUR => {
                        log::warn!("unimplemented opcode: {:?}", op);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // One operand codes
                    OpCode::BOA | OpCode::BSL | OpCode::FOB | OpCode::NUM | OpCode::DNA |
                    OpCode::MPp | OpCode::SKm | OpCode::SKp |
                    OpCode::UNJ | OpCode::MPJ | OpCode::XX1 | OpCode::SIL | OpCode::SOU |
                    OpCode::SSS | OpCode::ACH => {
                        let par_a = read_cur_varint(&mut cursor)?;

                        log::warn!("unimplemented opcode: {:?} {}", op, par_a);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // Two operand codes
                    OpCode::SKJ | OpCode::SMP | OpCode::PSp => {
                        let par_a = read_cur_varint(&mut cursor)?;
                        let par_b = read_cur_varint(&mut cursor)?;

                        log::warn!("unimplemented opcode: {:?} {} {}", op, par_a, par_b);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // Three operand codes
                    OpCode::TAM => {
                        let par_a = read_cur_varint(&mut cursor)?;
                        let par_b = read_cur_varint(&mut cursor)?;
                        let par_c = read_cur_varint(&mut cursor)?;

                        log::warn!("unimplemented opcode: {:?} {} {} {}", op, par_a, par_b, par_c);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // Four operand codes
                    OpCode::SNP => {
                        let par_a = read_cur_varint(&mut cursor)?;
                        let par_b = read_cur_varint(&mut cursor)?;
                        let par_c = read_cur_varint(&mut cursor)?;
                        let par_d = read_cur_varint(&mut cursor)?;

                        log::warn!("unimplemented opcode: {:?} {} {} {} {}", op, par_a, par_b, par_c, par_d);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                }
            } else {
                exec_state = TextScriptExecutionState::Ended;
            }
        } else {
            return Ok(TextScriptExecutionState::Ended);
        }

        if tick_npc != 0 {
            if let Some(npc) = game_scene.npc_map.npcs.get(&tick_npc) {
                npc.borrow_mut().tick(state, &mut game_scene.player)?;
            }
        }

        Ok(exec_state)
    }
}

pub struct TextScript {
    event_map: HashMap<u16, Vec<u8>>,
}

impl Clone for TextScript {
    fn clone(&self) -> Self {
        Self {
            event_map: self.event_map.clone(),
        }
    }
}

impl Default for TextScript {
    fn default() -> Self {
        TextScript::new()
    }
}

impl TextScript {
    pub fn new() -> TextScript {
        Self {
            event_map: HashMap::new(),
        }
    }

    /// Loads, decrypts and compiles a text script from specified stream.
    pub fn load_from<R: io::Read>(mut data: R) -> GameResult<TextScript> {
        let mut buf = Vec::new();
        data.read_to_end(&mut buf)?;

        let half = buf.len() / 2;
        let key = if let Some(0) = buf.get(half) {
            0xf9
        } else {
            (-(*buf.get(half).unwrap() as isize)) as u8
        };

        for (idx, byte) in buf.iter_mut().enumerate() {
            if idx == half {
                continue;
            }

            *byte = byte.wrapping_add(key);
        }

        TextScript::compile(&buf, false)
    }

    pub fn get_event_ids(&self) -> Vec<u16> {
        self.event_map.keys().copied().sorted().collect_vec()
    }

    /// Compiles a decrypted text script data into internal bytecode.
    pub fn compile(data: &[u8], strict: bool) -> GameResult<TextScript> {
        log::info!("data: {}", String::from_utf8_lossy(data));

        let mut event_map = HashMap::new();
        let mut iter = data.iter().copied().peekable();
        let mut last_event = 0;

        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' => {
                    iter.next();
                    let event_num = TextScript::read_number(&mut iter)? as u16;
                    TextScript::skip_until(b'\n', &mut iter)?;
                    last_event = event_num;

                    if event_map.contains_key(&event_num) {
                        if strict {
                            return Err(ParseError(format!("Event {} has been defined twice.", event_num)));
                        }

                        match TextScript::skip_until(b'#', &mut iter).ok() {
                            Some(_) => { continue; }
                            None => { break; }
                        }
                    }

                    let bytecode = TextScript::compile_event(&mut iter, strict, TextScriptEncoding::ShiftJIS)?;
                    log::info!("Successfully compiled event #{} ({} bytes generated).", event_num, bytecode.len());
                    event_map.insert(event_num, bytecode);
                }
                b'\r' | b'\n' | b' ' | b'\t' => {
                    iter.next();
                }
                n => {
                    // CS+ boss rush is the buggiest shit ever.
                    if !strict && last_event == 0 {
                        iter.next();
                        continue;
                    }

                    return Err(ParseError(format!("Unexpected token in event {}: {}", last_event, n as char)));
                }
            }
        }

        Ok(TextScript {
            event_map
        })
    }

    fn compile_event<I: Iterator<Item=u8>>(iter: &mut Peekable<I>, strict: bool, encoding: TextScriptEncoding) -> GameResult<Vec<u8>> {
        let mut bytecode = Vec::new();
        let mut char_buf = Vec::with_capacity(16);

        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' => {
                    if !char_buf.is_empty() {
                        TextScript::put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    // some events end without <END marker.
                    TextScript::put_varint(OpCode::_END as i32, &mut bytecode);
                    break;
                }
                b'<' => {
                    if !char_buf.is_empty() {
                        TextScript::put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    iter.next();
                    let n = iter.next_tuple::<(u8, u8, u8)>()
                        .map(|t| [t.0, t.1, t.2])
                        .ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;

                    let code = String::from_utf8_lossy(&n);

                    TextScript::compile_code(code.as_ref(), strict, iter, &mut bytecode)?;
                }
                _ => {
                    char_buf.push(chr);

                    iter.next();
                }
            }
        }

        Ok(bytecode)
    }

    fn put_string(buffer: &mut Vec<u8>, out: &mut Vec<u8>, encoding: TextScriptEncoding) {
        let mut cursor: Cursor<&Vec<u8>> = Cursor::new(buffer);
        let mut tmp_buf = Vec::new();
        let mut remaining = buffer.len() as u32;
        let mut chars = 0;

        while remaining > 0 {
            let (consumed, chr) = if encoding == TextScriptEncoding::UTF8 {
                read_cur_wtf8(&mut cursor, remaining)
            } else {
                read_cur_shift_jis(&mut cursor, remaining)
            };
            remaining -= consumed;
            chars += 1;

            TextScript::put_varint(chr as i32, &mut tmp_buf);
        }

        buffer.clear();

        TextScript::put_varint(OpCode::_STR as i32, out);
        TextScript::put_varint(chars, out);
        out.append(&mut tmp_buf);
    }

    fn put_varint(val: i32, out: &mut Vec<u8>) {
        let mut x = ((val as u32) >> 31) ^ ((val as u32) << 1);

        loop {
            let mut n = (x & 0x7f) as u8;
            x >>= 7;

            if x != 0 {
                n |= 0x80;
            }

            out.push(n);

            if x == 0 { break; }
        }
    }

    fn read_varint<I: Iterator<Item=u8>>(iter: &mut I) -> GameResult<i32> {
        let mut result = 0u32;

        for o in 0..5 {
            let n = iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
            result |= (n as u32 & 0x7f) << (o * 7);

            if n & 0x80 == 0 { break; }
        }

        Ok(((result << 31) ^ (result >> 1)) as i32)
    }

    fn compile_code<I: Iterator<Item=u8>>(code: &str, strict: bool, iter: &mut Peekable<I>, out: &mut Vec<u8>) -> GameResult {
        let instr = OpCode::from_str(code).map_err(|_| ParseError(format!("Unknown opcode: {}", code)))?;

        match instr {
            // Zero operand codes
            OpCode::AEp | OpCode::CAT | OpCode::CIL | OpCode::CLO | OpCode::CLR | OpCode::CPS |
            OpCode::CRE | OpCode::CSS | OpCode::END | OpCode::ESC | OpCode::FLA | OpCode::FMU |
            OpCode::FRE | OpCode::HMC | OpCode::INI | OpCode::KEY | OpCode::LDP | OpCode::MLP |
            OpCode::MM0 | OpCode::MNA | OpCode::MS2 | OpCode::MS3 | OpCode::MSG | OpCode::NOD |
            OpCode::PRI | OpCode::RMU | OpCode::SAT | OpCode::SLP | OpCode::SMC | OpCode::SPS |
            OpCode::STC | OpCode::SVP | OpCode::TUR | OpCode::WAS | OpCode::ZAM => {
                TextScript::put_varint(instr as i32, out);
            }
            // One operand codes
            OpCode::BOA | OpCode::BSL | OpCode::FOB | OpCode::FOM | OpCode::QUA | OpCode::UNI |
            OpCode::MYB | OpCode::MYD | OpCode::FAI | OpCode::FAO | OpCode::WAI | OpCode::FAC |
            OpCode::GIT | OpCode::NUM | OpCode::DNA | OpCode::DNP | OpCode::FLm | OpCode::FLp |
            OpCode::MPp | OpCode::SKm | OpCode::SKp | OpCode::EQp | OpCode::EQm | OpCode::MLp |
            OpCode::ITp | OpCode::ITm | OpCode::AMm | OpCode::UNJ | OpCode::MPJ | OpCode::YNJ |
            OpCode::EVE | OpCode::XX1 | OpCode::SIL | OpCode::LIp | OpCode::SOU | OpCode::CMU |
            OpCode::SSS | OpCode::ACH => {
                let operand = TextScript::read_number(iter)?;
                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand as i32, out);
            }
            // Two operand codes
            OpCode::FON | OpCode::MOV | OpCode::AMp | OpCode::NCJ | OpCode::ECJ | OpCode::FLJ |
            OpCode::ITJ | OpCode::SKJ | OpCode::AMJ | OpCode::SMP | OpCode::PSp => {
                let operand_a = TextScript::read_number(iter)?;
                if strict { TextScript::expect_char(b':', iter)?; } else { iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?; }
                let operand_b = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
            }
            // Three operand codes
            OpCode::ANP | OpCode::CNP | OpCode::INP | OpCode::TAM | OpCode::CMP => {
                let operand_a = TextScript::read_number(iter)?;
                if strict { TextScript::expect_char(b':', iter)?; } else { iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?; }
                let operand_b = TextScript::read_number(iter)?;
                if strict { TextScript::expect_char(b':', iter)?; } else { iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?; }
                let operand_c = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
                TextScript::put_varint(operand_c as i32, out);
            }
            // Four operand codes
            OpCode::TRA | OpCode::MNP | OpCode::SNP => {
                let operand_a = TextScript::read_number(iter)?;
                if strict { TextScript::expect_char(b':', iter)?; } else { iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?; }
                let operand_b = TextScript::read_number(iter)?;
                if strict { TextScript::expect_char(b':', iter)?; } else { iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?; }
                let operand_c = TextScript::read_number(iter)?;
                if strict { TextScript::expect_char(b':', iter)?; } else { iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?; }
                let operand_d = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
                TextScript::put_varint(operand_c as i32, out);
                TextScript::put_varint(operand_d as i32, out);
            }
            _ => {
                TextScript::put_varint(OpCode::_UNI as i32, out);
                log::warn!("Unimplemented opcode: {:?}", instr);
            }
        }

        Ok(())
    }

    fn expect_char<I: Iterator<Item=u8>>(expect: u8, iter: &mut I) -> GameResult {
        let res = iter.next();

        match res {
            Some(n) if n == expect => {
                Ok(())
            }
            Some(n) => {
                Err(ParseError(format!("Expected {}, found {}", expect as char, n as char)))
            }
            None => {
                Err(ParseError(str!("Script unexpectedly ended.")))
            }
        }
    }

    fn skip_until<I: Iterator<Item=u8>>(expect: u8, iter: &mut Peekable<I>) -> GameResult {
        while let Some(&chr) = iter.peek() {
            if chr == expect {
                return Ok(());
            } else {
                iter.next();
            }
        }

        Err(ParseError(str!("Script unexpectedly ended.")))
    }

    /// Reads a 4 digit TSC formatted number from iterator.
    /// Intentionally does no '0'..'9' range checking, since it was often exploited by modders.
    fn read_number<I: Iterator<Item=u8>>(iter: &mut Peekable<I>) -> GameResult<i32> {
        Some(0)
            .and_then(|result| iter.next().map(|v| result + 1000 * v.wrapping_sub(b'0') as i32))
            .and_then(|result| iter.next().map(|v| result + 100 * v.wrapping_sub(b'0') as i32))
            .and_then(|result| iter.next().map(|v| result + 10 * v.wrapping_sub(b'0') as i32))
            .and_then(|result| iter.next().map(|v| result + v.wrapping_sub(b'0') as i32))
            .ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))
    }


    pub fn has_event(&self, id: u16) -> bool {
        self.event_map.contains_key(&id)
    }
}

#[test]
fn test_varint() {
    for n in -4000..=4000 {
        let mut out = Vec::new();
        TextScript::put_varint(n, &mut out);

        let result = TextScript::read_varint(&mut out.iter().copied()).unwrap();
        assert_eq!(result, n);
        let mut cur = Cursor::new(&out);
        let result = read_cur_varint(&mut cur).unwrap();
        assert_eq!(result, n);
    }
}
