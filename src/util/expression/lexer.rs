use std::{
    fmt::{Debug, Display, Formatter},
    iter::Peekable,
    str::Chars,
};

use super::token::{ArithmeticOp, SourceLocation, Token};

pub(super) struct Cursor<'a> {
    input: &'a str,
    iter: Peekable<Chars<'a>>,
    /// Character position in the input string
    pos: usize,
    location: SourceLocation,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &str) -> Cursor {
        Cursor { input, iter: input.chars().peekable(), pos: 0, location: SourceLocation { line: 1, column: 0 } }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    fn inc_pos(&mut self) {
        let c = self.iter.peek();
        if c == Some(&'\n') {
            self.location.line += 1;
            self.location.column = 0;
        } else {
            self.location.column += 1;
        }
        self.pos += 1;
    }

    pub fn next(&mut self) -> Option<char> {
        let c = self.iter.next()?;
        self.inc_pos();
        Some(c)
    }

    pub fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        let c = self.iter.peek()?;
        if func(c) {
            self.next()
        } else {
            None
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.iter.peek().copied()
    }
}

pub(super) struct Tokenizer<'a> {
    cursor: Cursor<'a>,
    pub tokens: Vec<(Token, SourceLocation)>,
}

impl Tokenizer<'_> {
    pub fn new(input: &str) -> Tokenizer {
        Tokenizer { cursor: Cursor::new(input), tokens: Vec::new() }
    }

    pub fn scan(&mut self) -> LexResult {
        while let Some(c) = self.cursor.peek() {
            self.scan_item(c)?;
        }

        Ok(())
    }

    fn next(&mut self) -> LexResult<char> {
        self.cursor.next().ok_or_else(|| LexError::UnexpectedEndOfInput)
    }

    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> LexResult<char> {
        self.cursor.next_if(func).ok_or_else(|| LexError::UnexpectedEndOfInput)
    }

    fn scan_item(&mut self, c: char) -> LexResult {
        match c {
            ' ' | '\t' | '\n' => {
                self.cursor.next();
                return Ok(());
            }
            '0'..='9' => self.number(),
            '+' | '-' | '*' | '/' | '%' => self.operator(),
            '(' | ')' => self.bracket(),
            _ => return self.unexpected(c),
        }
    }

    fn number(&mut self) -> LexResult {
        let mut value = String::new();
        let mut is_fractional = false;
        let loc = self.cursor.location;

        while let Ok(c) = self.next_if(|c| matches!(c, '0'..='9' | '.')) {
            if c == '.' {
                if is_fractional {
                    return self.unexpected(c);
                }
                is_fractional = true;
            }

            value.push(c);
        }

        self.tokens.push((Token::Number { value, is_fractional }, loc));

        Ok(())
    }

    fn operator(&mut self) -> LexResult {
        let loc = self.cursor.location;
        let c = self.next()?;
        match c {
            '+' => self.tokens.push((Token::ArithmeticOp(ArithmeticOp::Add), loc)),
            '-' => self.tokens.push((Token::ArithmeticOp(ArithmeticOp::Subtract), loc)),
            '*' => self.tokens.push((Token::ArithmeticOp(ArithmeticOp::Multiply), loc)),
            '/' => self.tokens.push((Token::ArithmeticOp(ArithmeticOp::Divide), loc)),
            '%' => self.tokens.push((Token::ArithmeticOp(ArithmeticOp::Modulus), loc)),
            _ => return self.unexpected(c),
        }
        Ok(())
    }

    fn bracket(&mut self) -> LexResult {
        let loc = self.cursor.location;
        let c = self.next()?;
        match c {
            '(' => self.tokens.push((Token::LeftParen, loc)),
            ')' => self.tokens.push((Token::RightParen, loc)),
            _ => return self.unexpected(c),
        }
        Ok(())
    }

    fn unexpected(&self, c: char) -> LexResult {
        let loc = self.cursor.location;
        Err(LexError::UnexpectedChar { c, loc })
    }
}

pub type LexResult<T = ()> = Result<T, LexError>;

pub enum LexError {
    UnexpectedChar { c: char, loc: SourceLocation },
    UnexpectedEndOfInput,
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnexpectedChar { c, loc } => {
                write!(f, "Unexpected character '{}' at {}", c, loc)
            }
            LexError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
        }
    }
}

impl Debug for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_expression(input: &str, result: Vec<Token>) {
        let mut tok = Tokenizer::new(input);
        tok.scan().unwrap();
        // println!("tokens: {:?}", tok.tokens);
        let tokens = tok.tokens.into_iter().map(|(t, _)| t).collect::<Vec<_>>();
        assert_eq!(tokens, result);
    }

    #[test]
    fn test_number() {
        test_expression("1", vec![Token::Number { value: "1".to_string(), is_fractional: false }]);
        test_expression("1.0", vec![Token::Number { value: "1.0".to_string(), is_fractional: true }]);
    }

    #[test]
    fn test_arithmetic_ops() {
        test_expression(
            "1 + 22 * 33 / 4444 + 55555",
            vec![
                Token::Number { value: "1".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Add),
                Token::Number { value: "22".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Multiply),
                Token::Number { value: "33".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Divide),
                Token::Number { value: "4444".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Add),
                Token::Number { value: "55555".to_string(), is_fractional: false },
            ],
        );

        test_expression(
            "1.0 + 2.0",
            vec![
                Token::Number { value: "1.0".to_string(), is_fractional: true },
                Token::ArithmeticOp(ArithmeticOp::Add),
                Token::Number { value: "2.0".to_string(), is_fractional: true },
            ],
        );
        test_expression(
            "2.0 - 3",
            vec![
                Token::Number { value: "2.0".to_string(), is_fractional: true },
                Token::ArithmeticOp(ArithmeticOp::Subtract),
                Token::Number { value: "3".to_string(), is_fractional: false },
            ],
        );
    }

    #[test]
    fn test_brackets() {
        test_expression(
            "(1 + 2)",
            vec![
                Token::LeftParen,
                Token::Number { value: "1".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Add),
                Token::Number { value: "2".to_string(), is_fractional: false },
                Token::RightParen,
            ],
        );
        test_expression(
            "2 * (3 + 4)",
            vec![
                Token::Number { value: "2".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Multiply),
                Token::LeftParen,
                Token::Number { value: "3".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Add),
                Token::Number { value: "4".to_string(), is_fractional: false },
                Token::RightParen,
            ],
        );
        test_expression(
            "1 + ((2 * 3) + 4)",
            vec![
                Token::Number { value: "1".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Add),
                Token::LeftParen,
                Token::LeftParen,
                Token::Number { value: "2".to_string(), is_fractional: false },
                Token::ArithmeticOp(ArithmeticOp::Multiply),
                Token::Number { value: "3".to_string(), is_fractional: false },
                Token::RightParen,
                Token::ArithmeticOp(ArithmeticOp::Add),
                Token::Number { value: "4".to_string(), is_fractional: false },
                Token::RightParen,
            ],
        );
    }
}
