#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    /// Unrecognised character or malformed numeric literal.
    #[error("lex error at byte {pos}: {msg}")]
    Lex { pos: usize, msg: String },

    /// Token stream did not match the grammar.
    #[error("parse error: {0}")]
    Parse(String),

    /// Divisor or modulus evaluated to exactly zero.
    #[error("division by zero")]
    DivisionByZero,

    /// Variable name has no binding in the current session.
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),

    /// Function name not found in the built-in table.
    #[error("undefined function '{0}'")]
    UndefinedFunction(String),

    /// Argument was outside a function's mathematical domain.
    #[error("domain error in {name}: {msg}")]
    Domain { name: &'static str, msg: String },

    /// Result overflowed to ±infinity.
    #[error("arithmetic overflow: result is infinite")]
    Overflow,

    /// Function called with the wrong number of arguments.
    #[error("'{name}' expects {expected} argument(s), got {got}")]
    ArgCount {
        name: &'static str,
        expected: usize,
        got: usize,
    },
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, CalcError>;
