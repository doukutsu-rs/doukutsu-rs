#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VMOpcode {
    // No operation
    NOP,
    // Pop the top value off the stack
    STPOP,
    // Loads a specified variable onto the stack
    STLD(u16),
    // Pushes a number onto the stack
    STPSH(f64),
    // Adds the top (int, int) from the stack
    ADDII,
    // Adds the top (float, float) from the stack
    ADDFF,
    // Subtracts the (int, int) from the stack
    SUBII,
    // Subtracts the (float, float) from the stack
    SUBFF,
    // Multiplies the top (int, int) from the stack
    MULII,
    // Multiplies the top (float, float) from the stack
    MULFF,
    // Divides the top (int, int) from the stack
    DIVII,
    // Divides the top (float, float) from the stack
    DIVFF,
    // Modulus of the top (int, int) from the stack
    MODII,
    // Modulus of the top (float, float) from the stack
    MODFF,
    // Calls a function with specified index and passes 1 argument from the stack
    CALL1(u16),
    // Calls a function with specified index and passes 2 arguments from the stack
    CALL2(u16),
    // Calls a function with specified index and passes 3 arguments from the stack
    CALL3(u16),

    // Intrinsics
    // f64(int) -> f64
    INFLOAT2INT,
    // int(f64) -> int
    ININT2FLOAT,
    // frandom() -> f64
    INFRAND,
    // irandom() -> int
    INIRAND,
    // floor(f64) -> f64
    INFLOOR,
    // ceil(f64) -> f64
    INCEIL,
    // round(f64) -> f64
    INROUND,
    // abs(f64) -> f64
    INABS,
    // min(f64, f64) -> f64
    INMIN,
    // max(f64, f64) -> f64
    INMAX,
    // sqrt(f64) -> f64
    INSQRT,
    // cos(f64) -> f64
    INCOS,
    // sin(f64) -> f64
    INSIN,
}

pub struct VM {
}