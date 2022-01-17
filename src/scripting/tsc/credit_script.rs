use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Seek, SeekFrom};

use num_traits::FromPrimitive;

use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::scripting::tsc::bytecode_utils::{put_varint, read_cur_varint};
use crate::scripting::tsc::encryption::decrypt_tsc;
use crate::scripting::tsc::opcodes::CreditOpCode;
use crate::shared_game_state::SharedGameState;

pub struct CreditScript {
    pub(in crate::scripting::tsc) labels: HashMap<u16, u32>,
    pub(in crate::scripting::tsc) bytecode: Vec<u8>,
}

impl Default for CreditScript {
    fn default() -> Self {
        let mut bytecode = Vec::new();
        put_varint(CreditOpCode::StopCredits as i32, &mut bytecode);

        CreditScript { labels: HashMap::new(), bytecode }
    }
}

impl CreditScript {
    /// Loads, decrypts and compiles a credit script from specified stream.
    pub fn load_from<R: io::Read>(mut data: R, constants: &EngineConstants) -> GameResult<CreditScript> {
        let mut buf = Vec::new();
        data.read_to_end(&mut buf)?;

        if constants.textscript.encrypted {
            decrypt_tsc(&mut buf);
        }

        CreditScript::compile(&buf, false, constants.textscript.encoding)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CreditScriptExecutionState {
    Ended,
    Running(u32),
    WaitTicks(u32, u16),
}

pub struct CreditScriptLine {
    pub pos_x: f32,
    pub pos_y: f32,
    pub cast_id: u16,
    pub text: String,
}

pub struct CreditScriptVM {
    pub state: CreditScriptExecutionState,
    pub lines: Vec<CreditScriptLine>,
    pub text_offset: f32,
    script: CreditScript,
}

impl CreditScriptVM {
    pub fn new() -> CreditScriptVM {
        CreditScriptVM {
            state: CreditScriptExecutionState::Ended,
            lines: Vec::new(),
            text_offset: 0.0,
            script: CreditScript::default(),
        }
    }

    pub fn set_script(&mut self, script: CreditScript) {
        self.reset();
        self.script = script;
    }

    pub fn start(&mut self) {
        self.reset();
        self.state = CreditScriptExecutionState::Running(0);
    }

    pub fn reset(&mut self) {
        self.lines.clear();
        self.text_offset = 0.0;
        self.state = CreditScriptExecutionState::Ended;
    }

    pub fn run(state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if state.creditscript_vm.state != CreditScriptExecutionState::Ended {
            for line in &mut state.creditscript_vm.lines {
                line.pos_y -= 0.5;
            }
        }

        state.creditscript_vm.lines.retain(|l| l.pos_y > -16.0);

        loop {
            match state.creditscript_vm.state {
                CreditScriptExecutionState::Ended => {
                    break;
                }
                CreditScriptExecutionState::Running(ip) => {
                    let mut cursor: Cursor<&[u8]> = Cursor::new(&state.creditscript_vm.script.bytecode);
                    cursor.seek(SeekFrom::Start(ip as u64))?;

                    let op: CreditOpCode = if let Some(op) = FromPrimitive::from_i32(
                        read_cur_varint(&mut cursor).unwrap_or_else(|_| CreditOpCode::StopCredits as i32),
                    ) {
                        op
                    } else {
                        state.creditscript_vm.reset();
                        return Ok(());
                    };

                    match op {
                        CreditOpCode::_NOP => {
                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                        CreditOpCode::StopCredits => {
                            state.creditscript_vm.state = CreditScriptExecutionState::Ended;
                        }
                        CreditOpCode::PushLine => {
                            let cast_id = read_cur_varint(&mut cursor)? as u16;
                            let text_len = read_cur_varint(&mut cursor)?;
                            let mut text = String::new();
                            text.reserve(text_len as usize);

                            for _ in 0..text_len {
                                let chr =
                                    std::char::from_u32(read_cur_varint(&mut cursor)? as u32).unwrap_or('\u{fffd}');
                                text.push(chr);
                            }

                            let line = CreditScriptLine {
                                pos_x: state.creditscript_vm.text_offset,
                                pos_y: 256.0,
                                cast_id,
                                text,
                            };

                            state.creditscript_vm.lines.push(line);
                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                        CreditOpCode::Wait => {
                            let ticks = read_cur_varint(&mut cursor)? as u16;

                            state.creditscript_vm.state =
                                CreditScriptExecutionState::WaitTicks(cursor.position() as u32, ticks);
                        }
                        CreditOpCode::ChangeXOffset => {
                            let offset = read_cur_varint(&mut cursor)?;

                            state.creditscript_vm.text_offset = offset as f32;
                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                        CreditOpCode::ChangeMusic => {
                            let song = read_cur_varint(&mut cursor)? as u16;

                            state.sound_manager.play_song(song as usize, &state.constants, &state.settings, ctx)?;

                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                        CreditOpCode::FadeMusic => {
                            // todo

                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                        CreditOpCode::JumpLabel => {
                            let label = read_cur_varint(&mut cursor)? as u16;

                            if let Some(target) = state.creditscript_vm.script.labels.get(&label) {
                                state.creditscript_vm.state = CreditScriptExecutionState::Running(*target);
                                continue;
                            }

                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                        CreditOpCode::JumpFlag => {
                            let flag = read_cur_varint(&mut cursor)? as u16;
                            let label = read_cur_varint(&mut cursor)? as u16;

                            if state.get_flag(flag as usize) {
                                if let Some(target) = state.creditscript_vm.script.labels.get(&label) {
                                    state.creditscript_vm.state = CreditScriptExecutionState::Running(*target);
                                    continue;
                                }
                            }

                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                        CreditOpCode::JumpPlayer2 => {
                            let _label = read_cur_varint(&mut cursor)? as u16;
                            // todo

                            state.creditscript_vm.state = CreditScriptExecutionState::Running(cursor.position() as u32);
                        }
                    }
                }
                CreditScriptExecutionState::WaitTicks(ip, ticks) => {
                    if ticks == 0 {
                        state.creditscript_vm.state = CreditScriptExecutionState::Running(ip);
                    } else if ticks != 9999 {
                        state.creditscript_vm.state = CreditScriptExecutionState::WaitTicks(ip, ticks - 1);
                        break;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
