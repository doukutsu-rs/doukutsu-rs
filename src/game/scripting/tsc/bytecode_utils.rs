use std::io::{Cursor, Read};

use crate::framework::error::GameError::ParseError;
use crate::framework::error::GameResult;
use crate::game::scripting::tsc::text_script::TextScriptEncoding;

pub fn put_varint(val: i32, out: &mut Vec<u8>) {
    let mut x = ((val as u32) >> 31) ^ ((val as u32) << 1);

    loop {
        let mut n = (x & 0x7f) as u8;
        x >>= 7;

        if x != 0 {
            n |= 0x80;
        }

        out.push(n);

        if x == 0 {
            break;
        }
    }
}

pub fn read_cur_varint(cursor: &mut Cursor<&[u8]>) -> GameResult<i32> {
    let mut result = 0u32;

    for o in 0..5 {
        let mut n = [0u8];
        cursor.read_exact(&mut n)?;
        let [n] = n;

        result |= (n as u32 & 0x7f) << (o * 7);

        if n & 0x80 == 0 {
            break;
        }
    }

    Ok(((result << 31) ^ (result >> 1)) as i32)
}

#[allow(unused)]
pub fn read_varint<I: Iterator<Item = u8>>(iter: &mut I) -> GameResult<i32> {
    let mut result = 0u32;

    for o in 0..5 {
        let n = iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
        result |= (n as u32 & 0x7f) << (o * 7);

        if n & 0x80 == 0 {
            break;
        }
    }

    Ok(((result << 31) ^ (result >> 1)) as i32)
}

pub fn put_string(buffer: &mut Vec<u8>, out: &mut Vec<u8>, encoding: TextScriptEncoding) {
    if buffer.is_empty() {
        return;
    }
    let mut chars_count = 0;

    let mut tmp_buf = Vec::new();

    let encoding = match encoding {
        TextScriptEncoding::ShiftJIS => encoding_rs::SHIFT_JIS,
        TextScriptEncoding::UTF8 => encoding_rs::UTF_8,
        TextScriptEncoding::UTF16BE => encoding_rs::UTF_16BE,
        TextScriptEncoding::UTF16LE => encoding_rs::UTF_16LE,
        TextScriptEncoding::ISO_2022_JP => encoding_rs::ISO_2022_JP,
        TextScriptEncoding::ISO_8859_2 => encoding_rs::ISO_8859_2,
        TextScriptEncoding::ISO_8859_3 => encoding_rs::ISO_8859_3,
        TextScriptEncoding::ISO_8859_4 => encoding_rs::ISO_8859_4,
        TextScriptEncoding::ISO_8859_5 => encoding_rs::ISO_8859_5,
        TextScriptEncoding::ISO_8859_6 => encoding_rs::ISO_8859_6,
        TextScriptEncoding::ISO_8859_7 => encoding_rs::ISO_8859_7,
        TextScriptEncoding::ISO_8859_8 => encoding_rs::ISO_8859_8,
        TextScriptEncoding::ISO_8859_8_I => encoding_rs::ISO_8859_8_I,
        TextScriptEncoding::ISO_8859_10 => encoding_rs::ISO_8859_10,
        TextScriptEncoding::ISO_8859_13 => encoding_rs::ISO_8859_13,
        TextScriptEncoding::ISO_8859_14 => encoding_rs::ISO_8859_14,
        TextScriptEncoding::ISO_8859_15 => encoding_rs::ISO_8859_15,
        TextScriptEncoding::ISO_8859_16 => encoding_rs::ISO_8859_16,
        TextScriptEncoding::KOI8_R => encoding_rs::KOI8_R,
        TextScriptEncoding::KOI8_U => encoding_rs::KOI8_U,
        TextScriptEncoding::MACINTOSH => encoding_rs::MACINTOSH,
        TextScriptEncoding::EUC_JP => encoding_rs::EUC_JP,
        TextScriptEncoding::EUC_KR => encoding_rs::EUC_KR,
        TextScriptEncoding::GB18030 => encoding_rs::GB18030,
        TextScriptEncoding::GBK => encoding_rs::GBK,
        TextScriptEncoding::BIG5 => encoding_rs::BIG5,
        TextScriptEncoding::WINDOWS_1250 => encoding_rs::WINDOWS_1250,
        TextScriptEncoding::WINDOWS_1251 => encoding_rs::WINDOWS_1251,
        TextScriptEncoding::WINDOWS_1252 => encoding_rs::WINDOWS_1252,
        TextScriptEncoding::WINDOWS_1253 => encoding_rs::WINDOWS_1253,
        TextScriptEncoding::WINDOWS_1254 => encoding_rs::WINDOWS_1254,
        TextScriptEncoding::WINDOWS_1255 => encoding_rs::WINDOWS_1255,
        TextScriptEncoding::WINDOWS_1256 => encoding_rs::WINDOWS_1256,
        TextScriptEncoding::WINDOWS_1257 => encoding_rs::WINDOWS_1257,
        TextScriptEncoding::WINDOWS_1258 => encoding_rs::WINDOWS_1258,
    };

    let decoded_text = encoding.decode_without_bom_handling(&buffer).0;
    for chr in decoded_text.chars() {
        chars_count += 1;
        put_varint(chr as _, &mut tmp_buf);
    }

    buffer.clear();

    put_varint(chars_count, out);
    out.append(&mut tmp_buf);
}

#[test]
fn test_varint() {
    for n in -4000..=4000 {
        let mut out = Vec::new();
        put_varint(n, &mut out);

        let result = read_varint(&mut out.iter().copied()).unwrap();
        assert_eq!(result, n);
        let mut cur: Cursor<&[u8]> = Cursor::new(&out);
        let result = read_cur_varint(&mut cur).unwrap();
        assert_eq!(result, n);
    }
}
