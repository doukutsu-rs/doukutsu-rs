use core::cmp;

use alloc::{string::String, vec::Vec};

use crate::error::{GameError, GameResult, IOErrorKind};

mod cursor;
pub use cursor::*;

// The following source code has been adapted from https://github.com/rust-lang/rust/blob/1.78.0/library/std/src/io/mod.rs

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

pub(crate) unsafe fn append_to_string<F>(buf: &mut String, f: F) -> GameResult<usize>
where
    F: FnOnce(&mut Vec<u8>) -> GameResult<usize>,
{
    let mut g = Guard { len: buf.len(), buf: buf.as_mut_vec() };
    let ret = f(g.buf);
    if alloc::str::from_utf8(&g.buf[g.len..]).is_err() {
        ret.and_then(|_| Err(GameError::IOError(IOErrorKind::InvalidUtf8Data)))
    } else {
        g.len = g.buf.len();
        ret
    }
}

pub(crate) fn default_read_exact<R: Read + ?Sized>(this: &mut R, mut buf: &mut [u8]) -> GameResult<()> {
    while !buf.is_empty() {
        match this.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                buf = &mut buf[n..];
            }
            Err(e) => return Err(e),
        }
    }
    if !buf.is_empty() {
        Err(GameError::IOError(IOErrorKind::UnexpectedEof))
    } else {
        Ok(())
    }
}

const DEFAULT_BUF_SIZE: usize = 4 * 1024;

pub(crate) fn default_read_to_end<R: Read + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> GameResult<usize> {
    let start_len = buf.len();
    let mut read_buf = [0u8; DEFAULT_BUF_SIZE];
    loop {
        match r.read(&mut read_buf) {
            Ok(0) => {
                return Ok(buf.len() - start_len);
            }
            Ok(n) => buf.extend_from_slice(&read_buf[..n]),
            Err(e) => return Err(e),
        }
    }
}

pub(crate) fn default_read_to_string<R: Read + ?Sized>(r: &mut R, buf: &mut String) -> GameResult<usize> {
    // Note that we do *not* call `r.read_to_end()` here. We are passing
    // `&mut Vec<u8>` (the raw contents of `buf`) into the `read_to_end`
    // method to fill it up. An arbitrary implementation could overwrite the
    // entire contents of the vector, not just append to it (which is what
    // we are expecting).
    //
    // To prevent extraneously checking the UTF-8-ness of the entire buffer
    // we pass it to our hardcoded `default_read_to_end` implementation which
    // we know is guaranteed to only read data into the end of the buffer.
    unsafe { append_to_string(buf, |b| default_read_to_end(r, b)) }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> GameResult<usize>;

    fn read_exact(&mut self, buf: &mut [u8]) -> GameResult<()> {
        default_read_exact(self, buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> GameResult<usize> {
        default_read_to_end(self, buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> GameResult<usize> {
        default_read_to_string(self, buf)
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    #[inline]
    fn read_u8(&mut self) -> GameResult<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_i8(&mut self) -> GameResult<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    #[inline]
    fn read_u16_le(&mut self) -> GameResult<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    #[inline]
    fn read_u16_be(&mut self) -> GameResult<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    #[inline]
    fn read_i16_le(&mut self) -> GameResult<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    #[inline]
    fn read_i16_be(&mut self) -> GameResult<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    #[inline]
    fn read_u32_le(&mut self) -> GameResult<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    #[inline]
    fn read_u32_be(&mut self) -> GameResult<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    #[inline]
    fn read_i32_le(&mut self) -> GameResult<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    #[inline]
    fn read_i32_be(&mut self) -> GameResult<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    #[inline]
    fn read_u64_le(&mut self) -> GameResult<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    #[inline]
    fn read_u64_be(&mut self) -> GameResult<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    #[inline]
    fn read_i64_le(&mut self) -> GameResult<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    #[inline]
    fn read_i64_be(&mut self) -> GameResult<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }

    #[inline]
    fn read_u128_le(&mut self) -> GameResult<u128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }

    #[inline]
    fn read_u128_be(&mut self) -> GameResult<u128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Ok(u128::from_be_bytes(buf))
    }

    #[inline]
    fn read_i128_le(&mut self) -> GameResult<i128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Ok(i128::from_le_bytes(buf))
    }

    #[inline]
    fn read_i128_be(&mut self) -> GameResult<i128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Ok(i128::from_be_bytes(buf))
    }

    #[inline]
    fn read_f32_le(&mut self) -> GameResult<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    #[inline]
    fn read_f32_be(&mut self) -> GameResult<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }

    #[inline]
    fn read_f64_le(&mut self) -> GameResult<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    #[inline]
    fn read_f64_be(&mut self) -> GameResult<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> GameResult<usize>;

    fn flush(&mut self) -> GameResult<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> GameResult<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(GameError::IOError(IOErrorKind::WriteZero));
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    #[inline]
    fn write_u8(&mut self, n: u8) -> GameResult<()> {
        self.write_all(&[n])
    }

    #[inline]
    fn write_i8(&mut self, n: i8) -> GameResult<()> {
        self.write_all(&[n as u8])
    }

    #[inline]
    fn write_u16_le(&mut self, n: u16) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u16_be(&mut self, n: u16) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_i16_le(&mut self, n: i16) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i16_be(&mut self, n: i16) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_u32_le(&mut self, n: u32) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u32_be(&mut self, n: u32) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_i32_le(&mut self, n: i32) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i32_be(&mut self, n: i32) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_u64_le(&mut self, n: u64) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u64_be(&mut self, n: u64) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_i64_le(&mut self, n: i64) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i64_be(&mut self, n: i64) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_u128_le(&mut self, n: u128) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u128_be(&mut self, n: u128) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_i128_le(&mut self, n: i128) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i128_be(&mut self, n: i128) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_f32_le(&mut self, n: f32) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_f32_be(&mut self, n: f32) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }

    #[inline]
    fn write_f64_le(&mut self, n: f64) -> GameResult<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_f64_be(&mut self, n: f64) -> GameResult<()> {
        self.write_all(&n.to_be_bytes())
    }
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> GameResult<u64>;

    fn rewind(&mut self) -> GameResult<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn stream_position(&mut self) -> GameResult<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> GameResult<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> GameResult<()> {
        if buf.len() > self.len() {
            return Err(GameError::IOError(IOErrorKind::UnexpectedEof));
        }
        let (a, b) = self.split_at(buf.len());

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> GameResult<usize> {
        let len = self.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(*self);
        *self = &self[len..];
        Ok(len)
    }
}

impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> GameResult<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> GameResult<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> GameResult<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> GameResult<()> {
        (**self).read_exact(buf)
    }
}
