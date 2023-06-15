use std::fmt::Write;
use std::io::Cursor;

use num_traits::FromPrimitive;

use crate::framework::error::GameError::InvalidValue;
use crate::framework::error::GameResult;
use crate::game::scripting::tsc::bytecode_utils::read_cur_varint;
use crate::game::scripting::tsc::opcodes::TSCOpCode;
use crate::game::scripting::tsc::text_script::TextScript;

impl TextScript {
    pub fn decompile_event(&self, id: u16) -> GameResult<String> {
        if let Some(bytecode) = self.event_map.get(&id) {
            let mut result = String::new();
            let mut cursor: Cursor<&[u8]> = Cursor::new(bytecode);

            while let Ok(op_num) = read_cur_varint(&mut cursor) {
                let op_maybe: Option<TSCOpCode> = FromPrimitive::from_i32(op_num);

                if let Some(op) = op_maybe {
                    match op {
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
                            writeln!(&mut result, "{:?}()", op).unwrap();
                        }
                        // One operand codes
                        TSCOpCode::BOA
                        | TSCOpCode::BSL
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
                        | TSCOpCode::UNJ
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
                            let par_a = read_cur_varint(&mut cursor)?;

                            writeln!(&mut result, "{:?}({})", op, par_a).unwrap();
                        }
                        // Two operand codes
                        TSCOpCode::FON
                        | TSCOpCode::FOB
                        | TSCOpCode::MOV
                        | TSCOpCode::AMp
                        | TSCOpCode::NCJ
                        | TSCOpCode::ECJ
                        | TSCOpCode::FLJ
                        | TSCOpCode::ITJ
                        | TSCOpCode::SKJ
                        | TSCOpCode::AMJ
                        | TSCOpCode::SMP
                        | TSCOpCode::PSp
                        | TSCOpCode::IpN
                        | TSCOpCode::FFm => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;

                            writeln!(&mut result, "{:?}({}, {})", op, par_a, par_b).unwrap();
                        }
                        // Three operand codes
                        TSCOpCode::ANP | TSCOpCode::CNP | TSCOpCode::INP | TSCOpCode::TAM | TSCOpCode::CMP | TSCOpCode::INJ => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;
                            let par_c = read_cur_varint(&mut cursor)?;

                            writeln!(&mut result, "{:?}({}, {}, {})", op, par_a, par_b, par_c).unwrap();
                        }
                        // Four operand codes
                        TSCOpCode::TRA | TSCOpCode::MNP | TSCOpCode::SNP => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;
                            let par_c = read_cur_varint(&mut cursor)?;
                            let par_d = read_cur_varint(&mut cursor)?;

                            writeln!(&mut result, "{:?}({}, {}, {}, {})", op, par_a, par_b, par_c, par_d).unwrap();
                        }
                        TSCOpCode::_STR => {
                            let len = read_cur_varint(&mut cursor)?;

                            write!(&mut result, "%string(len = {}, value = \"", len).unwrap();
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
                                        result.push_str(&chr.escape_unicode().to_string());
                                    }
                                    _ => {
                                        result.push(chr);
                                    }
                                }
                            }
                            result.push_str("\")\n");
                        }
                        TSCOpCode::_NOP => result.push_str("%no_op()\n"),
                        TSCOpCode::_UNI => result.push_str("%unimplemented()\n"),
                        TSCOpCode::_END => result.push_str("%end_marker()\n"),
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
