use std::iter::Peekable;

use crate::framework::error::GameError::ParseError;
use crate::framework::error::GameResult;

pub fn expect_char<I: Iterator<Item=u8>>(expect: u8, iter: &mut I) -> GameResult {
    let res = iter.next();

    match res {
        Some(n) if n == expect => Ok(()),
        Some(n) => Err(ParseError(format!("Expected {}, found {}", expect as char, n as char))),
        None => Err(ParseError("Script unexpectedly ended.".to_string())),
    }
}

pub fn skip_until<I: Iterator<Item=u8>>(expect: u8, iter: &mut Peekable<I>) -> GameResult {
    while let Some(&chr) = iter.peek() {
        if chr == expect {
            return Ok(());
        } else {
            iter.next();
        }
    }

    Err(ParseError("Script unexpectedly ended.".to_string()))
}

/// Reads a 4 digit TSC formatted number from iterator.
/// Intentionally does no '0'..'9' range checking, since it was often exploited by modders.
pub fn read_number<I: Iterator<Item=u8>>(iter: &mut Peekable<I>) -> GameResult<i32> {
    Some(0)
        .and_then(|result| iter.next().map(|v| result + 1000 * v.wrapping_sub(b'0') as i32))
        .and_then(|result| iter.next().map(|v| result + 100 * v.wrapping_sub(b'0') as i32))
        .and_then(|result| iter.next().map(|v| result + 10 * v.wrapping_sub(b'0') as i32))
        .and_then(|result| iter.next().map(|v| result + v.wrapping_sub(b'0') as i32))
        .ok_or_else(|| ParseError("Script unexpectedly ended.".to_string()))
}
