use std::collections::HashMap;
use std::io;
use std::iter::Peekable;
use std::slice::Iter;
use std::str::FromStr;

use itertools::Itertools;

use crate::ggez::GameError::ParseError;
use crate::ggez::GameResult;
use crate::str;

/// Engine's text script VM operation codes.
#[derive(EnumString, Debug)]
pub enum OpCode {
    // ---- Internal opcodes (used by bytecode, no TSC representation)
    /// internal: no operation
    _NOP,
    /// internal: unimplemented
    _UNI,

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

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum TextScriptEncoding {
    UTF8,
    ShiftJIS,
}

enum TextScriptExecutionState {
    Ended,
    Running(u32, u32),
}

pub struct TextScriptManager {
    global_script: TextScript,
    scene_script: TextScript,
    state: TextScriptExecutionState,
}

pub struct TextScript {
    event_map: HashMap<u16, Vec<u8>>,
}

impl TextScript {
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

        TextScript::compile(&buf)
    }

    /// Compiles a decrypted text script data into internal bytecode.
    pub fn compile(data: &[u8]) -> GameResult<TextScript> {
        println!("data: {}", String::from_utf8(data.to_vec())?);

        let mut event_map = HashMap::new();
        let mut iter = data.iter().peekable();
        while let Some(&&chr) = iter.peek() {
            match chr {
                b'#' => {
                    iter.next();
                    let event_num = TextScript::read_number(&mut iter)? as u16;
                    TextScript::skip_until(b'\n', &mut iter)?;

                    if event_map.contains_key(&event_num) {
                        return Err(ParseError(format!("Event {} has been defined twice.", event_num)));
                    }

                    let bytecode = TextScript::compile_event(&mut iter)?;
                    log::info!("Successfully compiled event #{} ({} bytes generated).", event_num, bytecode.len());
                    println!("{:x?}", &bytecode);
                    event_map.insert(event_num, bytecode);
                }
                b'\r' | b'\n' => {
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

    fn compile_event(iter: &mut Peekable<Iter<u8>>) -> GameResult<Vec<u8>> {
        let mut bytecode = Vec::new();

        let mut char_buf = Vec::with_capacity(16);

        while let Some(&&chr) = iter.peek() {
            match chr {
                b'#' => {
                    if !char_buf.is_empty() {
                        TextScript::put_varint(char_buf.len() as i32, &mut bytecode);
                        bytecode.append(&mut char_buf);
                    }

                    // some events end without <END marker.
                    break;
                }
                b'<' => {
                    if !char_buf.is_empty() {
                        TextScript::put_varint(char_buf.len() as i32, &mut bytecode);
                        bytecode.append(&mut char_buf);
                    }

                    iter.next();
                    let n = iter.next_tuple::<(&u8, &u8, &u8)>()
                        .map(|t| [*t.0, *t.1, *t.2])
                        .ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;

                    let code = unsafe { std::str::from_utf8_unchecked(&n) };

                    TextScript::compile_code(code, iter, &mut bytecode)?;
                }
                _ => {
                    char_buf.push(chr);

                    iter.next();
                }
            }
        }

        Ok(bytecode)
    }

    fn put_varint(val: i32, out: &mut Vec<u8>) {
        let mut x = ((val as u32) >> 31) ^ ((val as u32) << 1);

        while x > 0x80 {
            out.push((x & 0x7f) as u8 | 0x80);
            x >>= 7;
        }

        out.push(x as u8);
    }

    fn read_varint(iter: &mut Peekable<Iter<u8>>) -> GameResult<i32> {
        let mut result = 0u32;

        for o in 0..5 {
            let &n = iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
            result |= (n as u32 & 0x7f) << (o * 7);

            if n & 0x80 == 0 {
                break;
            }
        }

        Ok(((result << 31) ^ (result >> 1)) as i32)
    }

    fn compile_code(code: &str, iter: &mut Peekable<Iter<u8>>, out: &mut Vec<u8>) -> GameResult {
        let instr = OpCode::from_str(code).map_err(|e| ParseError(format!("Unknown opcode: {}", code)))?;

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
                TextScript::expect_char(b':', iter)?;
                let operand_b = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
            }
            // Three operand codes
            OpCode::ANP | OpCode::CNP | OpCode::INP | OpCode::TAM | OpCode::CMP => {
                let operand_a = TextScript::read_number(iter)?;
                TextScript::expect_char(b':', iter)?;
                let operand_b = TextScript::read_number(iter)?;
                TextScript::expect_char(b':', iter)?;
                let operand_c = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
                TextScript::put_varint(operand_c as i32, out);
            }
            // Four operand codes
            OpCode::TRA | OpCode::MNP | OpCode::SNP => {
                let operand_a = TextScript::read_number(iter)?;
                TextScript::expect_char(b':', iter)?;
                let operand_b = TextScript::read_number(iter)?;
                TextScript::expect_char(b':', iter)?;
                let operand_c = TextScript::read_number(iter)?;
                TextScript::expect_char(b':', iter)?;
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

    fn expect_newline(iter: &mut Peekable<Iter<u8>>) -> GameResult {
        if let Some(b'\r') = iter.peek() {
            iter.next();
        }

        TextScript::expect_char(b'\n', iter)
    }

    fn expect_char(expect: u8, iter: &mut Peekable<Iter<u8>>) -> GameResult {
        let mut res = iter.next();

        match res {
            Some(&n) if n == expect => {
                Ok(())
            }
            Some(&n) => {
                Err(ParseError(format!("Expected {}, found {}", expect as char, n as char)))
            }
            None => {
                Err(ParseError(str!("Script unexpectedly ended.")))
            }
        }
    }

    fn skip_until(expect: u8, iter: &mut Peekable<Iter<u8>>) -> GameResult {
        while let Some(&chr) = iter.next() {
            if chr == expect {
                return Ok(());
            }
        }

        Err(ParseError(str!("Script unexpectedly ended.")))
    }

    /// Reads a 4 digit TSC formatted number from iterator.
    /// Intentionally does no '0'..'9' range checking, since it was often exploited by modders.
    fn read_number(iter: &mut Peekable<Iter<u8>>) -> GameResult<i32> {
        Some(0)
            .and_then(|result| iter.next().map(|&v| result + 1000 * (v - b'0') as i32))
            .and_then(|result| iter.next().map(|&v| result + 100 * (v - b'0') as i32))
            .and_then(|result| iter.next().map(|&v| result + 10 * (v - b'0') as i32))
            .and_then(|result| iter.next().map(|&v| result + (v - b'0') as i32))
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
        let result = TextScript::read_varint(&mut out.iter().peekable()).unwrap();
        assert_eq!(result, n);
    }
}
