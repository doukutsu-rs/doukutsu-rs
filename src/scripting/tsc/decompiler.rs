use std::io::Cursor;

use num_traits::FromPrimitive;

use crate::framework::error::GameError::InvalidValue;
use crate::framework::error::GameResult;
use crate::scripting::tsc::bytecode_utils::read_cur_varint;
use crate::scripting::tsc::opcodes::OpCode;
use crate::scripting::tsc::text_script::TextScript;

impl TextScript {
    pub fn decompile_event(&self, id: u16) -> GameResult<String> {
        if let Some(bytecode) = self.event_map.get(&id) {
            let mut result = String::new();
            let mut cursor = Cursor::new(bytecode);

            while let Ok(op_num) = read_cur_varint(&mut cursor) {
                let op_maybe: Option<OpCode> = FromPrimitive::from_i32(op_num);

                if let Some(op) = op_maybe {
                    match op {
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
                            result.push_str(format!("{:?}()\n", op).as_str());
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
                            let par_a = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({})\n", op, par_a).as_str());
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
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({}, {})\n", op, par_a, par_b).as_str());
                        }
                        // Three operand codes
                        OpCode::ANP | OpCode::CNP | OpCode::INP | OpCode::TAM | OpCode::CMP | OpCode::INJ => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;
                            let par_c = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({}, {}, {})\n", op, par_a, par_b, par_c).as_str());
                        }
                        // Four operand codes
                        OpCode::TRA | OpCode::MNP | OpCode::SNP => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;
                            let par_c = read_cur_varint(&mut cursor)?;
                            let par_d = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({}, {}, {}, {})\n", op, par_a, par_b, par_c, par_d).as_str());
                        }
                        OpCode::_STR => {
                            let len = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("%string(len = {}, value = \"", len).as_str());
                            for _ in 0..len {
                                let chr = std::char::from_u32(read_cur_varint(&mut cursor)? as u32).unwrap_or('?');
                                match chr {
                                    '\n' => {
                                        result.push_str("\\n");
                                    }
                                    '\r' => {
                                        result.push_str("\\r");
                                    }
                                    '\t' => {
                                        result.push_str("\\t");
                                    }
                                    '\u{0000}'..='\u{001f}' | '\u{0080}'..='\u{ffff}' => {
                                        result.push_str(chr.escape_unicode().to_string().as_str());
                                    }
                                    _ => {
                                        result.push(chr);
                                    }
                                }
                            }
                            result.push_str("\")\n");
                        }
                        OpCode::_NOP => result.push_str("%no_op()\n"),
                        OpCode::_UNI => result.push_str("%unimplemented()\n"),
                        OpCode::_END => result.push_str("%end_marker()\n"),
                    }
                } else {
                    break;
                }
            }

            Ok(result)
        } else {
            Err(InvalidValue("Unknown script.".to_string()))
        }
    }
}
