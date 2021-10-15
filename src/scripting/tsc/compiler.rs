use std::collections::HashMap;
use std::iter::Peekable;
use std::str::FromStr;

use itertools::Itertools;

use crate::encoding::read_cur_wtf8;
use crate::framework::error::GameError::ParseError;
use crate::framework::error::GameResult;
use crate::scripting::tsc::bytecode_utils::{put_string, put_varint};
use crate::scripting::tsc::opcodes::OpCode;
use crate::scripting::tsc::parse_utils::{expect_char, read_number, skip_until};
use crate::scripting::tsc::text_script::{TextScript, TextScriptEncoding};

impl TextScript {
    /// Compiles a decrypted text script data into internal bytecode.
    pub fn compile(data: &[u8], strict: bool, encoding: TextScriptEncoding) -> GameResult<TextScript> {
        let mut event_map = HashMap::new();
        let mut iter = data.iter().copied().peekable();
        let mut last_event = 0;

        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' => {
                    iter.next();
                    let event_num = read_number(&mut iter)? as u16;
                    if iter.peek().is_some() {
                        skip_until(b'\n', &mut iter)?;
                        iter.next();
                    }
                    last_event = event_num;

                    if event_map.contains_key(&event_num) {
                        if strict {
                            return Err(ParseError(format!("Event {} has been defined twice.", event_num)));
                        }

                        match skip_until(b'#', &mut iter).ok() {
                            Some(_) => {
                                continue;
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    let bytecode = TextScript::compile_event(&mut iter, strict, encoding)?;
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

        Ok(TextScript { event_map })
    }

    fn compile_event<I: Iterator<Item = u8>>(
        iter: &mut Peekable<I>,
        strict: bool,
        encoding: TextScriptEncoding,
    ) -> GameResult<Vec<u8>> {
        let mut bytecode = Vec::new();
        let mut char_buf = Vec::with_capacity(16);
        let mut allow_next_event = true;

        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' if allow_next_event => {
                    if !char_buf.is_empty() {
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    // some events end without <END marker.
                    put_varint(OpCode::_END as i32, &mut bytecode);
                    break;
                }
                b'<' => {
                    allow_next_event = false;
                    if char_buf.len() > 2 {
                        if let Some(&c) = char_buf.last() {
                            if c == b'\n' {
                                let _ = char_buf.pop();
                            }
                        }
                    }

                    if !char_buf.is_empty() {
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    iter.next();
                    let n = iter
                        .next_tuple::<(u8, u8, u8)>()
                        .map(|t| [t.0, t.1, t.2])
                        .ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;

                    let code = String::from_utf8_lossy(&n);

                    TextScript::compile_code(code.as_ref(), strict, iter, &mut bytecode)?;
                }
                b'\r' => {
                    iter.next();
                }
                b'\n' => {
                    allow_next_event = true;
                    char_buf.push(chr);

                    iter.next();
                }
                _ => {
                    allow_next_event = false;
                    char_buf.push(chr);

                    iter.next();
                }
            }
        }

        Ok(bytecode)
    }

    fn compile_code<I: Iterator<Item = u8>>(
        code: &str,
        strict: bool,
        iter: &mut Peekable<I>,
        out: &mut Vec<u8>,
    ) -> GameResult {
        let instr = OpCode::from_str(code).map_err(|_| ParseError(format!("Unknown opcode: {}", code)))?;

        match instr {
            // Zero operand codes
            OpCode::AEp
            | OpCode::CAT
            | OpCode::CIL
            | OpCode::CLO
            | OpCode::CLR
            | OpCode::CPS
            | OpCode::CRE
            | OpCode::CSS
            | OpCode::END
            | OpCode::ESC
            | OpCode::FLA
            | OpCode::FMU
            | OpCode::FRE
            | OpCode::HMC
            | OpCode::INI
            | OpCode::KEY
            | OpCode::LDP
            | OpCode::MLP
            | OpCode::MM0
            | OpCode::MNA
            | OpCode::MS2
            | OpCode::MS3
            | OpCode::MSG
            | OpCode::NOD
            | OpCode::PRI
            | OpCode::RMU
            | OpCode::SAT
            | OpCode::SLP
            | OpCode::SMC
            | OpCode::SPS
            | OpCode::STC
            | OpCode::SVP
            | OpCode::TUR
            | OpCode::WAS
            | OpCode::ZAM
            | OpCode::HM2
            | OpCode::POP
            | OpCode::KE2
            | OpCode::FR2 => {
                put_varint(instr as i32, out);
            }
            // One operand codes
            OpCode::BOA
            | OpCode::BSL
            | OpCode::FOB
            | OpCode::FOM
            | OpCode::QUA
            | OpCode::UNI
            | OpCode::MYB
            | OpCode::MYD
            | OpCode::FAI
            | OpCode::FAO
            | OpCode::WAI
            | OpCode::FAC
            | OpCode::GIT
            | OpCode::NUM
            | OpCode::DNA
            | OpCode::DNP
            | OpCode::FLm
            | OpCode::FLp
            | OpCode::MPp
            | OpCode::SKm
            | OpCode::SKp
            | OpCode::EQp
            | OpCode::EQm
            | OpCode::MLp
            | OpCode::ITp
            | OpCode::ITm
            | OpCode::AMm
            | OpCode::UNJ
            | OpCode::MPJ
            | OpCode::YNJ
            | OpCode::EVE
            | OpCode::XX1
            | OpCode::SIL
            | OpCode::LIp
            | OpCode::SOU
            | OpCode::CMU
            | OpCode::SSS
            | OpCode::ACH
            | OpCode::S2MV
            | OpCode::S2PJ
            | OpCode::PSH => {
                let operand = read_number(iter)?;
                put_varint(instr as i32, out);
                put_varint(operand as i32, out);
            }
            // Two operand codes
            OpCode::FON
            | OpCode::MOV
            | OpCode::AMp
            | OpCode::NCJ
            | OpCode::ECJ
            | OpCode::FLJ
            | OpCode::ITJ
            | OpCode::SKJ
            | OpCode::AMJ
            | OpCode::SMP
            | OpCode::PSp
            | OpCode::IpN
            | OpCode::FFm => {
                let operand_a = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_b = read_number(iter)?;

                put_varint(instr as i32, out);
                put_varint(operand_a as i32, out);
                put_varint(operand_b as i32, out);
            }
            // Three operand codes
            OpCode::ANP | OpCode::CNP | OpCode::INP | OpCode::TAM | OpCode::CMP | OpCode::INJ => {
                let operand_a = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_b = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_c = read_number(iter)?;

                put_varint(instr as i32, out);
                put_varint(operand_a as i32, out);
                put_varint(operand_b as i32, out);
                put_varint(operand_c as i32, out);
            }
            // Four operand codes
            OpCode::TRA | OpCode::MNP | OpCode::SNP => {
                let operand_a = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_b = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_c = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_d = read_number(iter)?;

                put_varint(instr as i32, out);
                put_varint(operand_a as i32, out);
                put_varint(operand_b as i32, out);
                put_varint(operand_c as i32, out);
                put_varint(operand_d as i32, out);
            }
            OpCode::_NOP | OpCode::_UNI | OpCode::_STR | OpCode::_END => {
                unreachable!()
            }
        }

        Ok(())
    }
}
