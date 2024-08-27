use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Not,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BitwiseOp {
    And,
    Or,
    Xor,
    Not,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Number { value: String, is_fractional: bool },
    ArithmeticOp(ArithmeticOp),
    LogicalOp(LogicalOp),
    ComparisonOp(ComparisonOp),
    BitwiseOp(BitwiseOp),
    LeftParen,
    RightParen,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}
