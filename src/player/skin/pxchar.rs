use std::io::{Cursor, Read};

use byteorder::{ReadBytesExt, LE};
use num_traits::{AsPrimitive, Num};

use crate::framework::error::GameResult;
use crate::framework::error::GameError::{ResourceLoadError, InvalidValue};
use crate::framework::filesystem;
use crate::framework::context::Context;

struct PxCharReader<T: AsRef<[u8]>> {
    cursor: Cursor<T>,
    bit_ptr: usize,
}

impl<T: AsRef<[u8]>> PxCharReader<T> {
    fn read_integer_shifted<O>(&mut self, bits: usize) -> GameResult<O>
    where
        O: 'static + Num + Copy,
        u128: AsPrimitive<O>,
        u64: AsPrimitive<O>,
        u32: AsPrimitive<O>,
        u16: AsPrimitive<O>,
        u8: AsPrimitive<O>,
    {
        let shift = self.bit_ptr & 7;
        self.cursor.set_position((self.bit_ptr / 8) as u64);
        self.bit_ptr += bits;

        match bits {
            // fast paths for aligned bit sizes
            0 => Ok((0u8).as_()),
            8 if shift == 0 => Ok(self.cursor.read_u8()?.as_()),
            16 if shift == 0 => Ok(self.cursor.read_u16::<LE>()?.as_()),
            24 if shift == 0 => Ok(self.cursor.read_u24::<LE>()?.as_()),
            32 if shift == 0 => Ok(self.cursor.read_u32::<LE>()?.as_()),
            48 if shift == 0 => Ok(self.cursor.read_u48::<LE>()?.as_()),
            64 if shift == 0 => Ok(self.cursor.read_u64::<LE>()?.as_()),
            128 if shift == 0 => Ok(self.cursor.read_u128::<LE>()?.as_()),
            // paths for bit shifted numbers
            1..=8 => Ok(((self.cursor.read_u16::<LE>()? >> shift) & ((1 << bits) - 1)).as_()),
            9..=16 => Ok(((self.cursor.read_u24::<LE>()? >> shift) & ((1 << bits) - 1)).as_()),
            17..=24 => Ok(((self.cursor.read_u32::<LE>()? >> shift) & ((1 << bits) - 1)).as_()),
            25..=40 => Ok(((self.cursor.read_u48::<LE>()? >> shift) & ((1 << bits) - 1)).as_()),
            41..=56 => Ok(((self.cursor.read_u64::<LE>()? >> shift) & ((1 << bits) - 1)).as_()),
            57..=120 => Ok(((self.cursor.read_u128::<LE>()? >> shift) & ((1 << bits) - 1)).as_()),
            121..=128 => {
                let mut result = self.cursor.read_u128::<LE>()? >> shift;
                result |= (self.cursor.read_u8()? as u128) << (128 - shift);
                Ok(result.as_())
            }
            _ => Err(InvalidValue("Cannot read integers bigger than 128 bits.".to_owned())),
        }
    }

    fn read_ranged<O>(&mut self, max_value: u32) -> GameResult<O>
    where
        O: 'static + Num + Copy,
        u128: AsPrimitive<O>,
        u64: AsPrimitive<O>,
        u32: AsPrimitive<O>,
        u16: AsPrimitive<O>,
        u8: AsPrimitive<O>,
    {
        self.read_integer_shifted((32 - max_value.next_power_of_two().leading_zeros()) as usize)
    }

    fn read_string(&mut self, max_length: u32) -> GameResult<String> {
        let mut output = Vec::new();

        let length = self.read_ranged::<u32>(max_length)?;
        output.reserve(length as usize);

        for _ in 0..length {
            output.push(self.read_integer_shifted::<u8>(8)?)
        }

        Ok(String::from_utf8_lossy(&output).to_string())
    }
}

impl<T: AsRef<[u8]>> Read for PxCharReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // align to byte
        if self.bit_ptr & 7 != 0 {
            self.bit_ptr = (self.bit_ptr + 7) & !7;
            self.cursor.read_u8()?;
        }

        let result = self.cursor.read(buf);
        self.bit_ptr = (self.cursor.position() * 8) as usize;
        result
    }
}

pub struct PxChar {}

impl PxChar {
    pub fn load_pxchar(path: &str, ctx: &mut Context) -> GameResult<PxChar> {
        let mut reader = PxCharReader {
            cursor: Cursor::new({
                let mut stream = filesystem::open(ctx, path)?;
                let mut data = Vec::new();
                stream.read_to_end(&mut data)?;
                data
            }),
            bit_ptr: 0,
        };

        let mut magic_buf = [0u8; 6];
        reader.read_exact(&mut magic_buf)?;

        if &magic_buf != b"PXCHAR" {
            return Err(ResourceLoadError("Invalid magic number.".to_string()));
        }

        let version = reader.read_u8()?;
        if version > 5 {
            return Err(ResourceLoadError("Unsupported version.".to_string()));
        }

        let string = reader.read_string(0x100)?;
        println!("{}", string);
        let description = reader.read_string(0x100)?;
        println!("{}", description);

        Ok(PxChar {})
    }
}

#[test]
fn test() {
    use crate::framework::filesystem::mount_vfs;
    use crate::framework::vfs::PhysicalFS;

    let mut ctx = crate::framework::context::Context::new();
    mount_vfs(&mut ctx, Box::new(PhysicalFS::new("data".as_ref(), true)));

    println!("lol");
    PxChar::load_pxchar("/Player.pxchar", &mut ctx).unwrap();
}
