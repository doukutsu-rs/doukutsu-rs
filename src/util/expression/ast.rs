#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Float(f64),
    Integer(i64),
    BinaryExpr { left: Box<Node>, op: BinaryOp, right: Box<Node> },
    UnaryExpr { op: UnaryOp, expr: Box<Node> },
}
