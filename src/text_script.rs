use std::io;

use crate::ggez::GameResult;

/// Engine's internal text script VM operation codes.
/// Based on https://www.cavestory.org/guides/basicmodding/guide/tscnotes.htm and game reverse engineering.
pub enum OpCode {
    // ---- Internal opcodes (used by bytecode, no TSC representation)
    /// internal: no operation
    NOP,

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

    FLm,
    FLp,
    MPp,
    SKm,
    SKp,

    EQp,
    EQm,
    MLp,
    ITp,
    ITm,
    AMp,
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
    PSp,
    SLP,
    ZAM,

    AEp,
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

    // ---- Custom opcodes, for use by modders ----
}

pub struct TextScript {}

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

        let tsc = TextScript {};
        Ok(tsc)
    }
}
