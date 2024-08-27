use std::fmt::{Debug, Display};

use super::{
    ast::{BinaryOp, Node, UnaryOp},
    token::{ArithmeticOp, SourceLocation, Token},
};

pub struct Parser {
    ast: Vec<Node>,
    tokens: Vec<(Token, SourceLocation)>,
    pos: usize,
}

// -- the grammar --
// expression -> term
// term -> factor ( ('+' | '-') factor )*
// factor -> unary ( ('*' | '/' | '%') unary )*
// unary -> ('!' | '-') unary | primary
// primary -> NUMBER | '(' expression ')'

impl Parser {
    pub fn new(tokens: Vec<(Token, SourceLocation)>) -> Parser {
        Parser { tokens, pos: 0, ast: Vec::new() }
    }

    fn current(&self) -> ParseResult<&(Token, SourceLocation)> {
        self.tokens.get(self.pos).ok_or_else(|| ParseError::UnexpectedEnd)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn expect(&self, token: Token) -> bool {
        !self.is_end() && self.current().map(|(t, _)| t == &token).unwrap_or(false)
    }

    fn parse_expr(&mut self) -> ParseResult<Node> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> ParseResult<Node> {
        let mut node = self.parse_factor()?;

        while !self.is_end() {
            let op = if let (Token::ArithmeticOp(op), _) = self.current()? {
                match op {
                    ArithmeticOp::Add => BinaryOp::Add,
                    ArithmeticOp::Subtract => BinaryOp::Subtract,
                    _ => break,
                }
            } else {
                break;
            };

            self.advance();
            let right = self.parse_factor()?;
            node = Node::BinaryExpr { left: Box::new(node), op, right: Box::new(right) };
        }

        Ok(node)
    }

    fn parse_factor(&mut self) -> ParseResult<Node> {
        let mut node = self.parse_unary()?;

        while !self.is_end() {
            let op = if let (Token::ArithmeticOp(op), _) = self.current()? {
                match op {
                    ArithmeticOp::Multiply => BinaryOp::Multiply,
                    ArithmeticOp::Divide => BinaryOp::Divide,
                    ArithmeticOp::Modulus => BinaryOp::Modulus,
                    _ => break,
                }
            } else {
                break;
            };

            self.advance();
            let right = self.parse_unary()?;
            node = Node::BinaryExpr { left: Box::new(node), op, right: Box::new(right) };
        }

        Ok(node)
    }

    fn parse_unary(&mut self) -> ParseResult<Node> {
        let current = self.current()?;
        if let (Token::ArithmeticOp(ArithmeticOp::Subtract), _) = current {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Node::UnaryExpr { op: UnaryOp::Negate, expr: Box::new(expr) })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> ParseResult<Node> {
        if let (Token::Number { .. }, _) = self.current()? {
            let (current, loc) = self.current()?;
            let node = match current {
                Token::Number { value, is_fractional } => {
                    if *is_fractional {
                        Node::Float(value.parse().unwrap())
                    } else {
                        Node::Integer(value.parse().unwrap())
                    }
                }
                _ => return Err(ParseError::UnexpectedToken { token: current.clone(), loc: *loc }),
            };

            self.advance();
            Ok(node)
        } else if self.expect(Token::LeftParen) {
            self.advance();
            let node = self.parse_expr()?;
            if !self.expect(Token::RightParen) {
                return self.unexpected();
            }
            self.advance();
            Ok(node)
        } else {
            self.unexpected()
        }
    }

    fn unexpected<T>(&self) -> ParseResult<T> {
        let (token, loc) = self.current()?;
        Err(ParseError::UnexpectedToken { token: token.clone(), loc: loc.clone() })
    }

    pub fn parse(&mut self) -> ParseResult {
        let mut ast = Vec::new();
        while !self.is_end() {
            ast.push(self.parse_expr()?);
        }

        self.ast = ast;

        Ok(())
    }
}

type ParseResult<T = ()> = Result<T, ParseError>;

pub enum ParseError {
    UnexpectedToken { token: Token, loc: SourceLocation },
    UnexpectedEnd,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { token, loc } => {
                write!(f, "Unexpected token {:?} at {}", token, loc)
            }
            ParseError::UnexpectedEnd => write!(f, "Unexpected end of input"),
        }
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::expression::{ast::BinaryOp, lexer::Tokenizer};

    use super::*;

    fn parse(input: &str) -> Vec<Node> {
        let mut tokenizer = Tokenizer::new(input);
        tokenizer.scan().unwrap();

        let mut parser = Parser::new(tokenizer.tokens);
        parser.parse().unwrap();

        println!("{:#?}", parser.ast);
        parser.ast
    }

    #[test]
    fn test_parser() {
        parse("-4 + (2 * 3) % 5 + (2 + 2) * 2");

        assert_eq!(
            parse("1 + 2 * 3"),
            vec![Node::BinaryExpr {
                left: Box::new(Node::Integer(1)),
                op: BinaryOp::Add,
                right: Box::new(Node::BinaryExpr {
                    left: Box::new(Node::Integer(2)),
                    op: BinaryOp::Multiply,
                    right: Box::new(Node::Integer(3)),
                }),
            }]
        );
    }
}
