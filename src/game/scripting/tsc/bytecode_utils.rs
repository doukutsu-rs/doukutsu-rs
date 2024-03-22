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
        // auto detection
        _ => {
            let guess = charset_normalizer_rs::from_bytes(&buffer, None);
            let Some(gussed) = guess.get_best() else {
                return;
            };

            match gussed.encoding() {
                "iso-2022-jp" => encoding_rs::ISO_2022_JP,
                "iso-8859-2" => encoding_rs::ISO_8859_2,
                "iso-8859-3" => encoding_rs::ISO_8859_3,
                "iso-8859-4" => encoding_rs::ISO_8859_4,
                "iso-8859-5" => encoding_rs::ISO_8859_5,
                "iso-8859-6" => encoding_rs::ISO_8859_6,
                "iso-8859-7" => encoding_rs::ISO_8859_7,
                "iso-8859-8" => encoding_rs::ISO_8859_8,
                "iso-8859-8-i" => encoding_rs::ISO_8859_8_I,
                "iso-8859-10" => encoding_rs::ISO_8859_10,
                "iso-8859-13" => encoding_rs::ISO_8859_13,
                "iso-8859-14" => encoding_rs::ISO_8859_14,
                "iso-8859-15" => encoding_rs::ISO_8859_15,
                "iso-8859-16" => encoding_rs::ISO_8859_16,

                "koi8-r" => encoding_rs::KOI8_R,
                "koi8-u" => encoding_rs::KOI8_U,

                "macintosh" => encoding_rs::MACINTOSH,

                "euc-jp" => encoding_rs::EUC_JP,
                "euc-kr" => encoding_rs::EUC_KR,

                "gb18030" => encoding_rs::GB18030,
                "gbk" => encoding_rs::GBK,
                "big5" => encoding_rs::BIG5,

                "windows-1250" => encoding_rs::WINDOWS_1250,
                "windows-1251" => encoding_rs::WINDOWS_1251,
                "windows-1252" => encoding_rs::WINDOWS_1252,
                "windows-1253" => encoding_rs::WINDOWS_1253,
                "windows-1254" => encoding_rs::WINDOWS_1254,
                "windows-1255" => encoding_rs::WINDOWS_1255,
                "windows-1256" => encoding_rs::WINDOWS_1256,
                "windows-1257" => encoding_rs::WINDOWS_1257,
                "windows-1258" => encoding_rs::WINDOWS_1258,

                "utf-8" => encoding_rs::UTF_8,
                "utf-16be" => encoding_rs::UTF_16BE,
                "utf-16le" => encoding_rs::UTF_16LE,
                
                "x-mac-cyrillic" => encoding_rs::X_MAC_CYRILLIC,
                "x-user-defined" => encoding_rs::X_USER_DEFINED,
                _ => encoding_rs::UTF_8,
            }
        }
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
