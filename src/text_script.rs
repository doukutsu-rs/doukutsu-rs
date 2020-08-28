use std::collections::HashMap;
use std::io;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::iter::Peekable;
use std::str::FromStr;

use byteorder::ReadBytesExt;
use itertools::Itertools;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{SharedGameState, str};
use crate::bitfield;
use crate::common::{FadeState, FadeDirection};
use crate::ggez::{Context, GameResult};
use crate::ggez::GameError::ParseError;
use crate::scene::game_scene::GameScene;

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

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum TextScriptExecutionState {
    Ended,
    Running(u16, u32),
    Msg(u16, u32, u32, u8),
    WaitTicks(u16, u32, u32),
    WaitInput(u16, u32),
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

/// Decodes UTF-8 character in a less strict way.
/// http://simonsapin.github.io/wtf-8/#decoding-wtf-8
fn read_cur_wtf8(cursor: &mut Cursor<&Vec<u8>>, max_bytes: u32) -> (u32, char) {
    let result: u32;
    let consumed: u32;

    if max_bytes == 0 {
        return (0, '\u{fffd}');
    }

    match cursor.read_u8() {
        Ok(byte @ 0x00..=0x7f) => {
            consumed = 1;
            result = byte as u32;
        }
        Ok(byte @ 0xc2..=0xdf) if max_bytes >= 2 => {
            let byte2 = { if let Ok(n) = cursor.read_u8() { n } else { return (1, '\u{fffd}'); } };

            consumed = 2;
            result = (byte as u32 & 0x1f) << 6 | (byte2 as u32 & 0x3f);
        }
        Ok(byte @ 0xe0..=0xef) if max_bytes >= 3 => {
            let byte2 = { if let Ok(n) = cursor.read_u8() { n } else { return (1, '\u{fffd}'); } };
            let byte3 = { if let Ok(n) = cursor.read_u8() { n } else { return (2, '\u{fffd}'); } };

            consumed = 3;
            result = (byte as u32 & 0x0f) << 12 | (byte2 as u32 & 0x3f) << 6 | (byte3 as u32 & 0x3f);
        }
        Ok(byte @ 0xf0..=0xf4) if max_bytes >= 4 => {
            let byte2 = { if let Ok(n) = cursor.read_u8() { n } else { return (1, '\u{fffd}'); } };
            let byte3 = { if let Ok(n) = cursor.read_u8() { n } else { return (2, '\u{fffd}'); } };
            let byte4 = { if let Ok(n) = cursor.read_u8() { n } else { return (3, '\u{fffd}'); } };

            consumed = 4;
            result = (byte as u32 & 0x07) << 18 | (byte2 as u32 & 0x3f) << 12 | (byte3 as u32 & 0x3f) << 6 | (byte4 as u32 & 0x3f);
        }
        _ => { return (1, '\u{fffd}'); }
    }

    (consumed, std::char::from_u32(result).unwrap_or('\u{fffd}'))
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

    pub fn reset(&mut self) {
        self.state = TextScriptExecutionState::Ended;
        self.clear_text_box();
    }

    pub fn clear_text_box(&mut self) {
        self.flags.0 = 0;
        self.face = 0;
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
                    state.control_flags.set_flag_x04(false);
                    break;
                }
                TextScriptExecutionState::Running(event, ip) => {
                    state.control_flags.set_flag_x04(true);
                    state.textscript_vm.state = TextScriptVM::execute(event, ip, state, game_scene, ctx)?;

                    if state.textscript_vm.state == TextScriptExecutionState::Ended {
                        state.textscript_vm.reset();
                    }
                }
                TextScriptExecutionState::Msg(event, ip, remaining, timer) => {
                    if timer > 0 {
                        state.textscript_vm.state = TextScriptExecutionState::Msg(event, ip, remaining, timer - 1);
                        break;
                    }

                    if let Some(bytecode) = state.textscript_vm.scripts.find_script(event) {
                        let mut cursor = Cursor::new(bytecode);
                        cursor.seek(SeekFrom::Start(ip as u64))?;

                        let (consumed, chr) = read_cur_wtf8(&mut cursor, remaining);

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

                        if (remaining - consumed) > 0 {
                            let ticks = if state.key_state.jump() || state.key_state.fire() { 1 } else { 4 };
                            state.textscript_vm.state = TextScriptExecutionState::Msg(event, ip + consumed, remaining - consumed, ticks);
                        } else {
                            state.textscript_vm.state = TextScriptExecutionState::Running(event, ip + consumed);
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
                        let len = read_cur_varint(&mut cursor)? as u32;
                        if state.textscript_vm.flags.render() {
                            exec_state = TextScriptExecutionState::Msg(event, cursor.position() as u32, len, 4);
                        } else {
                            // simply skip the text if we aren't in message mode.
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32 + len);
                        }
                    }
                    OpCode::_END | OpCode::END => {
                        state.control_flags.set_flag_x01(true);
                        state.control_flags.set_control_enabled(true);

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
                    OpCode::WAI => {
                        let ticks = read_cur_varint(&mut cursor)? as u32;
                        exec_state = TextScriptExecutionState::WaitTicks(event, cursor.position() as u32, ticks);
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
                    OpCode::MSG => {
                        state.textscript_vm.face = 0;
                        state.textscript_vm.current_line = TextScriptLine::Line1;
                        state.textscript_vm.line_1.clear();
                        state.textscript_vm.line_2.clear();
                        state.textscript_vm.line_3.clear();
                        state.textscript_vm.flags.set_render(true);
                        state.textscript_vm.flags.set_background_visible(true);
                        state.textscript_vm.flags.set_flag_x10(state.textscript_vm.flags.flag_x40());
                        state.textscript_vm.flags.set_position_top(false);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CLO => {
                        state.textscript_vm.flags.set_render(false);
                        state.textscript_vm.flags.set_background_visible(false);
                        state.textscript_vm.flags.set_flag_x10(false);
                        state.textscript_vm.flags.set_position_top(false);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::TRA => {
                        let map_id = read_cur_varint(&mut cursor)? as usize;
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        let pos_x = read_cur_varint(&mut cursor)? as isize * 16 * 0x200;
                        let pos_y = read_cur_varint(&mut cursor)? as isize * 16 * 0x200;

                        let mut new_scene = GameScene::new(state, ctx, map_id)?;
                        new_scene.player = game_scene.player.clone();
                        new_scene.player.vel_x = 0;
                        new_scene.player.vel_y = 0;
                        new_scene.player.x = pos_x;
                        new_scene.player.y = pos_y;

                        state.textscript_vm.suspend = true;

                        state.next_scene = Some(Box::new(new_scene));
                        exec_state = TextScriptExecutionState::Running(event_num, 0);
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
                            state.fade_state = FadeState::FadeOut(-15, direction);
                        }

                        exec_state = TextScriptExecutionState::WaitFade(event, cursor.position() as u32);
                    }
                    // unimplemented opcodes
                    // Zero operands
                    OpCode::AEp | OpCode::CAT | OpCode::CIL | OpCode::CPS |
                    OpCode::CRE | OpCode::CSS | OpCode::ESC | OpCode::FLA | OpCode::FMU |
                    OpCode::HMC | OpCode::INI | OpCode::LDP | OpCode::MLP |
                    OpCode::MNA | OpCode::MS2 | OpCode::MS3 |
                    OpCode::RMU | OpCode::SAT | OpCode::SLP | OpCode::SMC | OpCode::SPS |
                    OpCode::STC | OpCode::SVP | OpCode::TUR | OpCode::WAS | OpCode::ZAM => {
                        log::warn!("unimplemented opcode: {:?}", op);
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // One operand codes
                    OpCode::BOA | OpCode::BSL | OpCode::FOB | OpCode::FOM | OpCode::QUA | OpCode::UNI |
                    OpCode::MYB | OpCode::MYD |
                    OpCode::GIT | OpCode::NUM | OpCode::DNA | OpCode::DNP |
                    OpCode::MPp | OpCode::SKm | OpCode::SKp | OpCode::EQp | OpCode::EQm |
                    OpCode::ITp | OpCode::ITm | OpCode::AMm | OpCode::UNJ | OpCode::MPJ | OpCode::YNJ |
                    OpCode::XX1 | OpCode::SIL | OpCode::LIp | OpCode::SOU | OpCode::CMU |
                    OpCode::SSS | OpCode::ACH => {
                        let par_a = read_cur_varint(&mut cursor)?;
                        log::warn!("unimplemented opcode: {:?} {}", op, par_a);
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // Two operand codes
                    OpCode::FON | OpCode::MOV | OpCode::AMp | OpCode::NCJ | OpCode::ECJ |
                    OpCode::ITJ | OpCode::SKJ | OpCode::AMJ | OpCode::SMP | OpCode::PSp => {
                        let par_a = read_cur_varint(&mut cursor)?;
                        let par_b = read_cur_varint(&mut cursor)?;
                        log::warn!("unimplemented opcode: {:?} {} {}", op, par_a, par_b);
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // Three operand codes
                    OpCode::ANP | OpCode::CNP | OpCode::INP | OpCode::TAM => {
                        let par_a = read_cur_varint(&mut cursor)?;
                        let par_b = read_cur_varint(&mut cursor)?;
                        let par_c = read_cur_varint(&mut cursor)?;
                        log::warn!("unimplemented opcode: {:?} {} {} {}", op, par_a, par_b, par_c);
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // Four operand codes
                    OpCode::MNP | OpCode::SNP => {
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
        let code = unsafe { std::str::from_utf8_unchecked(data) };
        println!("data: {}", code);

        let mut event_map = HashMap::new();
        let mut iter = data.iter().copied().peekable();
        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' => {
                    iter.next();
                    let event_num = TextScript::read_number(&mut iter)? as u16;
                    TextScript::skip_until(b'\n', &mut iter)?;

                    if event_map.contains_key(&event_num) {
                        if strict {
                            return Err(ParseError(format!("Event {} has been defined twice.", event_num)));
                        } else {
                            continue;
                        }
                    }

                    let bytecode = TextScript::compile_event(&mut iter, strict)?;
                    log::info!("Successfully compiled event #{} ({} bytes generated).", event_num, bytecode.len());
                    println!("{:x?}", &bytecode);
                    event_map.insert(event_num, bytecode);
                }
                b'\r' | b'\n' | b' ' => {
                    iter.next();
                }
                n => {
                    return Err(ParseError(format!("Unexpected token: {}", n as char)));
                }
            }
        }

        Ok(TextScript {
            event_map
        })
    }

    fn compile_event<I: Iterator<Item=u8>>(iter: &mut Peekable<I>, strict: bool) -> GameResult<Vec<u8>> {
        let mut bytecode = Vec::new();
        let mut char_buf = Vec::with_capacity(16);

        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' => {
                    if !char_buf.is_empty() {
                        TextScript::put_string(&mut char_buf, &mut bytecode);
                    }

                    // some events end without <END marker.
                    TextScript::put_varint(OpCode::_END as i32, &mut bytecode);
                    break;
                }
                b'<' => {
                    if !char_buf.is_empty() {
                        TextScript::put_string(&mut char_buf, &mut bytecode);
                    }

                    iter.next();
                    let n = iter.next_tuple::<(u8, u8, u8)>()
                        .map(|t| [t.0, t.1, t.2])
                        .ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;

                    let code = unsafe { std::str::from_utf8_unchecked(&n) };

                    TextScript::compile_code(code, strict, iter, &mut bytecode)?;
                }
                _ => {
                    char_buf.push(chr);

                    iter.next();
                }
            }
        }

        Ok(bytecode)
    }

    fn put_string(buffer: &mut Vec<u8>, out: &mut Vec<u8>) {
        TextScript::put_varint(OpCode::_STR as i32, out);
        TextScript::put_varint(buffer.len() as i32, out);
        out.append(buffer);
    }

    fn put_varint(val: i32, out: &mut Vec<u8>) {
        let mut x = ((val as u32) >> 31) ^ ((val as u32) << 1);

        while x > 0x80 {
            out.push((x & 0x7f) as u8 | 0x80);
            x >>= 7;
        }

        out.push(x as u8);
    }

    fn read_varint<I: Iterator<Item=u8>>(iter: &mut I) -> GameResult<i32> {
        let mut result = 0u32;

        for o in 0..5 {
            let n = iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
            result |= (n as u32 & 0x7f) << (o * 7);

            if n & 0x80 == 0 {
                break;
            }
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

    fn expect_newline<I: Iterator<Item=u8>>(iter: &mut Peekable<I>) -> GameResult {
        if let Some(b'\r') = iter.peek() {
            iter.next();
        }

        TextScript::expect_char(b'\n', iter)
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

    fn skip_until<I: Iterator<Item=u8>>(expect: u8, iter: &mut I) -> GameResult {
        for chr in iter {
            if chr == expect {
                return Ok(());
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
    for &n in [1_i32, 23, 456, 7890, 12345, -1, -23, -456].iter() {
        let mut out = Vec::new();
        TextScript::put_varint(n, &mut out);
        let result = TextScript::read_varint(&mut out.iter().copied()).unwrap();
        assert_eq!(result, n);
    }
}
