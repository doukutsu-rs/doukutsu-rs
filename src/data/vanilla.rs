use std::{
    env,
    io::{Read, Write},
    ops::Range,
    path::PathBuf,
};

use byteorder::{LE, WriteBytesExt};

use crate::data::exe_parser::ExeParser;
use crate::framework::{
    context::Context,
    error::{GameError::ParseError, GameResult},
    filesystem,
};

pub struct VanillaExtractor {
    exe_buffer: Vec<u8>,
    data_base_dir: String,
    root: PathBuf,
}

const VANILLA_STAGE_COUNT: u32 = 95;
const VANILLA_STAGE_ENTRY_SIZE: u32 = 0xC8;
const VANILLA_STAGE_TABLE_SIZE: u32 = VANILLA_STAGE_COUNT * VANILLA_STAGE_ENTRY_SIZE;

trait RangeExt {
    fn to_usize(&self) -> std::ops::Range<usize>;
}

impl RangeExt for Range<u32> {
    fn to_usize(&self) -> std::ops::Range<usize> {
        (self.start as usize)..(self.end as usize)
    }
}

impl VanillaExtractor {
    pub fn from(ctx: &mut Context, exe_name: String, data_base_dir: String) -> Option<Self> {
        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        let mut vanilla_exe_path = env::current_dir().unwrap();

        #[cfg(target_os = "android")]
        let mut vanilla_exe_path = PathBuf::from(sdl2::filesystem::pref_path(crate::common::ORG_NAME, crate::common::APP_NAME).unwrap());

        #[cfg(target_os = "horizon")]
        let mut vanilla_exe_path = PathBuf::from("sdmc:/switch/doukutsu-rs/");

        vanilla_exe_path.push(&exe_name);

        log::info!("Looking for vanilla game executable at {:?}", vanilla_exe_path);

        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        if !vanilla_exe_path.is_file() {
            vanilla_exe_path = env::current_exe().unwrap();
            vanilla_exe_path.pop();
            vanilla_exe_path.push(&exe_name);
        }

        if !vanilla_exe_path.is_file() {
            return None;
        }

        let mut root = vanilla_exe_path.clone();
        root.pop();

        log::info!("Found vanilla game executable, attempting to extract resources.");

        if filesystem::exists(ctx, "/stage.sect") {
            log::info!("Vanilla resources are already extracted, not proceeding.");
            return None;
        }

        let file = std::fs::File::open(vanilla_exe_path);
        if file.is_err() {
            log::error!("Failed to open vanilla game executable: {}", file.unwrap_err());
            return None;
        }

        let mut exe_buffer = Vec::new();
        let result = file.unwrap().read_to_end(&mut exe_buffer);
        if result.is_err() {
            log::error!("Failed to read vanilla game executable: {}", result.unwrap_err());
            return None;
        }

        Some(Self { exe_buffer, data_base_dir, root })
    }

    pub fn extract_data(&self) -> GameResult {
        let parser = ExeParser::from(&self.exe_buffer);
        if parser.is_err() {
            return Err(ParseError("Failed to create vanilla parser.".to_string()));
        }

        let parser = parser.unwrap();

        self.extract_organya(&parser)?;
        self.extract_bitmaps(&parser)?;
        self.extract_stage_table(&parser)?;

        Ok(())
    }

    fn deep_create_dir_if_not_exists(&self, path: PathBuf) -> GameResult {
        if path.is_dir() {
            return Ok(());
        }

        let result = std::fs::create_dir_all(path);
        if result.is_err() {
            return Err(ParseError(format!("Failed to create directory structure: {}", result.unwrap_err())));
        }

        Ok(())
    }

    fn extract_organya(&self, parser: &ExeParser) -> GameResult {
        let orgs = parser.get_resource_dir("ORG".to_string());

        if orgs.is_err() {
            return Err(ParseError("Failed to retrieve Organya resource directory.".to_string()));
        }

        for org in orgs.unwrap().data_files {
            let mut org_path = self.root.clone();
            org_path.push(self.data_base_dir.clone());
            org_path.push("Org/");

            if self.deep_create_dir_if_not_exists(org_path.clone()).is_err() {
                return Err(ParseError("Failed to create directory structure.".to_string()));
            }

            org_path.push(format!("{}.org", org.name));

            let mut org_file = match std::fs::File::create(org_path) {
                Ok(file) => file,
                Err(_) => {
                    return Err(ParseError("Failed to create organya file.".to_string()));
                }
            };

            let result = org_file.write_all(&org.bytes);
            if result.is_err() {
                return Err(ParseError("Failed to write organya file.".to_string()));
            }

            log::info!("Extracted organya file: {}", org.name);
        }

        Ok(())
    }

    fn extract_bitmaps(&self, parser: &ExeParser) -> GameResult {
        let bitmaps = parser.get_bitmap_dir();

        if bitmaps.is_err() {
            return Err(ParseError("Failed to retrieve bitmap directory.".to_string()));
        }

        for bitmap in bitmaps.unwrap().data_files {
            let mut data_path = self.root.clone();
            data_path.push(self.data_base_dir.clone());

            if self.deep_create_dir_if_not_exists(data_path.clone()).is_err() {
                return Err(ParseError("Failed to create data directory structure.".to_string()));
            }

            data_path.push(format!("{}.pbm", bitmap.name));

            let file = std::fs::File::create(data_path);
            if file.is_err() {
                return Err(ParseError("Failed to create bitmap file.".to_string()));
            }

            let mut file = file.unwrap();

            file.write_u8(0x42)?; // B
            file.write_u8(0x4D)?; // M
            file.write_u32::<LE>(bitmap.bytes.len() as u32 + 0xE)?; // Size of BMP file
            file.write_u32::<LE>(0)?; // unused null bytes
            file.write_u32::<LE>(0x76)?; // Bitmap data offset (hardcoded for now, might wanna get the actual offset)

            let result = file.write_all(&bitmap.bytes);
            if result.is_err() {
                return Err(ParseError("Failed to write bitmap file.".to_string()));
            }

            log::info!("Extracted bitmap file: {}", bitmap.name);
        }

        Ok(())
    }

    fn find_stage_table_offset(&self, parser: &ExeParser) -> Option<Range<u32>> {
        let range = parser.get_named_section_byte_range(".csmap".to_string());
        if range.is_err() {
            return None;
        }

        let pattern = [
            // add     esp, 8
            0x83u8, 0xc4, 0x08,
            // mov     eax, [ebp+arg_0]
            0x8b, 0x45, 0x08,
            // imul    eax, 0C8h
            0x69, 0xc0, 0xc8, 0x00, 0x00, 0x00,
            // add     eax, offset gTMT
            0x05, // 0x??, 0x??, 0x??, 0x??
        ];

        let text = parser.section_headers.by_name(".text")?;
        let text_range = text.file_range().to_usize();
        let text_range_start = text_range.start;
        let offset = self.exe_buffer[text_range]
            .windows(pattern.len())
            .position(|window| window == pattern)?;
        let offset = text_range_start + offset;
        let offset = u32::from_le_bytes([
            self.exe_buffer[offset + 13],
            self.exe_buffer[offset + 14],
            self.exe_buffer[offset + 15],
            self.exe_buffer[offset + 16],
        ]);
        log::info!("Found stage table offset: 0x{:X}", offset);

        let section = parser.section_headers.by_rva(offset - parser.image_base)?;
        let offset_inside_range = offset.checked_sub(section.VirtualAddress + parser.image_base)?;
        let range = section.file_range();
        
        let data_start = range.start + offset_inside_range;
        let data_end = data_start + VANILLA_STAGE_TABLE_SIZE;
        Some(data_start..data_end)
    }

    fn extract_stage_table(&self, parser: &ExeParser) -> GameResult {
        let range = self.find_stage_table_offset(parser);
        let range = match range {
            Some(range) => range,
            None => return Err(ParseError("Failed to retrieve stage table from executable.".to_string())),
        };
        let range = range.to_usize();

        let byte_slice = &self.exe_buffer[range];

        let mut stage_tbl_path = self.root.clone();
        stage_tbl_path.push(self.data_base_dir.clone());

        if self.deep_create_dir_if_not_exists(stage_tbl_path.clone()).is_err() {
            return Err(ParseError("Failed to create data directory structure.".to_string()));
        }

        stage_tbl_path.push("stage.sect");

        let mut stage_tbl_file = match std::fs::File::create(stage_tbl_path) {
            Ok(file) => file,
            Err(_) => {
                return Err(ParseError("Failed to create stage table file.".to_string()));
            }
        };

        let result = stage_tbl_file.write_all(byte_slice);
        if result.is_err() {
            return Err(ParseError("Failed to write to stage table file.".to_string()));
        }

        Ok(())
    }
}
