use std::collections::HashMap;
use std::iter::Peekable;
use std::str::FromStr;

use itertools::Itertools;

use crate::framework::error::GameError::ParseError;
use crate::framework::error::GameResult;
use crate::scripting::tsc::bytecode_utils::{put_string, put_varint};
use crate::scripting::tsc::credit_script::CreditScript;
use crate::scripting::tsc::opcodes::{CreditOpCode, TSCOpCode};
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
                        put_varint(TSCOpCode::_STR as i32, &mut bytecode);
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    // some events end without <END marker.
                    put_varint(TSCOpCode::_END as i32, &mut bytecode);
                    break;
                }
                b'<' => {
                    allow_next_event = false;

                    if !char_buf.is_empty() {
                        put_varint(TSCOpCode::_STR as i32, &mut bytecode);
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    iter.next();
                    let n = iter
                        .next_tuple::<(u8, u8, u8)>()
                        .map(|t| [t.0, t.1, t.2])
                        .ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;

                    let code = String::from_utf8_lossy(&n);

                    TextScript::compile_code(&code, strict, iter, &mut bytecode)?;
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
                    char_buf.push(chr);

                    iter.next();
                }
            }
        }

        // Some nicalis challenges are very broken
        if !strict {
            put_varint(TSCOpCode::_END as i32, &mut bytecode);
        }

        Ok(bytecode)
    }

    fn compile_code<I: Iterator<Item = u8>>(
        code: &str,
        strict: bool,
        iter: &mut Peekable<I>,
        out: &mut Vec<u8>,
    ) -> GameResult {
        let instr = TSCOpCode::from_str(code).map_err(|_| ParseError(format!("Unknown opcode: {}", code)))?;

        match instr {
            // Zero operand codes
            TSCOpCode::AEp
            | TSCOpCode::CAT
            | TSCOpCode::CIL
            | TSCOpCode::CLO
            | TSCOpCode::CLR
            | TSCOpCode::CPS
            | TSCOpCode::CRE
            | TSCOpCode::CSS
            | TSCOpCode::END
            | TSCOpCode::ESC
            | TSCOpCode::FLA
            | TSCOpCode::FMU
            | TSCOpCode::FRE
            | TSCOpCode::HMC
            | TSCOpCode::INI
            | TSCOpCode::KEY
            | TSCOpCode::LDP
            | TSCOpCode::MLP
            | TSCOpCode::MM0
            | TSCOpCode::MNA
            | TSCOpCode::MS2
            | TSCOpCode::MS3
            | TSCOpCode::MSG
            | TSCOpCode::NOD
            | TSCOpCode::PRI
            | TSCOpCode::RMU
            | TSCOpCode::SAT
            | TSCOpCode::SLP
            | TSCOpCode::SMC
            | TSCOpCode::SPS
            | TSCOpCode::STC
            | TSCOpCode::SVP
            | TSCOpCode::TUR
            | TSCOpCode::WAS
            | TSCOpCode::ZAM
            | TSCOpCode::HM2
            | TSCOpCode::POP
            | TSCOpCode::KE2
            | TSCOpCode::FR2 => {
                put_varint(instr as i32, out);
            }
            // One operand codes
            TSCOpCode::BOA
            | TSCOpCode::BSL
            | TSCOpCode::FOB
            | TSCOpCode::FOM
            | TSCOpCode::QUA
            | TSCOpCode::UNI
            | TSCOpCode::MYB
            | TSCOpCode::MYD
            | TSCOpCode::FAI
            | TSCOpCode::FAO
            | TSCOpCode::WAI
            | TSCOpCode::FAC
            | TSCOpCode::GIT
            | TSCOpCode::NUM
            | TSCOpCode::DNA
            | TSCOpCode::DNP
            | TSCOpCode::FLm
            | TSCOpCode::FLp
            | TSCOpCode::MPp
            | TSCOpCode::SKm
            | TSCOpCode::SKp
            | TSCOpCode::EQp
            | TSCOpCode::EQm
            | TSCOpCode::MLp
            | TSCOpCode::ITp
            | TSCOpCode::ITm
            | TSCOpCode::AMm
            | TSCOpCode::MPJ
            | TSCOpCode::YNJ
            | TSCOpCode::EVE
            | TSCOpCode::XX1
            | TSCOpCode::SIL
            | TSCOpCode::LIp
            | TSCOpCode::SOU
            | TSCOpCode::CMU
            | TSCOpCode::SSS
            | TSCOpCode::ACH
            | TSCOpCode::S2MV
            | TSCOpCode::S2PJ
            | TSCOpCode::PSH => {
                let operand = read_number(iter)?;
                put_varint(instr as i32, out);
                put_varint(operand as i32, out);
            }
            // Two operand codes
            TSCOpCode::FON
            | TSCOpCode::MOV
            | TSCOpCode::AMp
            | TSCOpCode::NCJ
            | TSCOpCode::ECJ
            | TSCOpCode::FLJ
            | TSCOpCode::ITJ
            | TSCOpCode::SKJ
            | TSCOpCode::AMJ
            | TSCOpCode::UNJ
            | TSCOpCode::SMP
            | TSCOpCode::PSp
            | TSCOpCode::IpN
            | TSCOpCode::FFm => {
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
            TSCOpCode::ANP | TSCOpCode::CNP | TSCOpCode::INP | TSCOpCode::TAM | TSCOpCode::CMP | TSCOpCode::INJ => {
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
            TSCOpCode::TRA | TSCOpCode::MNP | TSCOpCode::SNP => {
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
            TSCOpCode::_NOP | TSCOpCode::_UNI | TSCOpCode::_STR | TSCOpCode::_END => {
                unreachable!()
            }
        }

        Ok(())
    }
}

impl CreditScript {
    pub fn compile(data: &[u8], strict: bool, encoding: TextScriptEncoding) -> GameResult<CreditScript> {
        let mut labels = HashMap::new();
        let mut bytecode = Vec::new();
        let mut iter = data.iter().copied().peekable();

        while let Some(chr) = iter.next() {
            match chr {
                b'/' => {
                    put_varint(CreditOpCode::StopCredits as i32, &mut bytecode);
                }
                b'[' => {
                    let mut char_buf = Vec::new();

                    while let Some(&chr) = iter.peek() {
                        if chr == b']' {
                            iter.next();
                            break;
                        }

                        char_buf.push(chr);
                        iter.next();
                    }

                    if let Ok(cast_tile) = read_number(&mut iter) {
                        put_varint(CreditOpCode::PushLine as i32, &mut bytecode);
                        put_varint((cast_tile as u16) as i32, &mut bytecode);
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }
                }
                b'-' => {
                    let ticks = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::Wait as i32, &mut bytecode);
                    put_varint(ticks as i32, &mut bytecode);
                }
                b'+' => {
                    let offset = read_number(&mut iter)?;

                    put_varint(CreditOpCode::ChangeXOffset as i32, &mut bytecode);
                    put_varint(offset, &mut bytecode);
                }
                b'!' => {
                    let music = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::ChangeMusic as i32, &mut bytecode);
                    put_varint(music as i32, &mut bytecode);
                }
                b'~' => {
                    put_varint(CreditOpCode::FadeMusic as i32, &mut bytecode);
                }
                b'l' => {
                    let label = read_number(&mut iter)? as u16;
                    let pos = bytecode.len() as u32;

                    labels.insert(label, pos);
                }
                b'j' => {
                    let label = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::JumpLabel as i32, &mut bytecode);
                    put_varint(label as i32, &mut bytecode);
                }
                b'f' => {
                    let flag = read_number(&mut iter)? as u16;
                    if strict {
                        expect_char(b':', &mut iter)?;
                    } else {
                        iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                    }
                    let label = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::JumpFlag as i32, &mut bytecode);
                    put_varint(flag as i32, &mut bytecode);
                    put_varint(label as i32, &mut bytecode);
                }
                b'p' => {
                    iter.next(); // idfk what's that for, in cs+ Credits.tsc it's '2'.

                    if strict {
                        expect_char(b':', &mut iter)?;
                    } else {
                        iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                    }

                    let label = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::JumpPlayer2 as i32, &mut bytecode);
                    put_varint(label as i32, &mut bytecode);
                }
                _ => (),
            }
        }

        Ok(CreditScript { labels, bytecode })
    }
}
