use crate::error::{CalcError, Result};
use crate::lexer::Token;

/// Infix binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

/// Abstract syntax tree node.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Numeric literal.
    Number(f64),
    /// Variable or constant reference.
    Var(String),
    /// Variable assignment: `name = rhs`.
    Assign(String, Box<Expr>),
    /// Infix binary expression.
    BinOp {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
    /// Unary negation.
    Neg(Box<Expr>),
    /// Function call.
    Call { name: String, args: Vec<Expr> },
}

/// Top-down operator precedence (Pratt) parser.
///
/// Binding-power table
/// ```text
///   +  -   →  (1, 2)   left-assoc
///   *  /  %  →  (3, 4)   left-assoc
///   ^        →  (6, 5)   right-assoc
///   unary -  →  prefix bp 9
/// ```
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a parser from the token list produced by [`crate::lexer::Lexer`].
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    #[inline]
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    #[inline]
    fn peek_ahead(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or(&Token::Eof)
    }

    #[inline]
    fn bump(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<()> {
        if self.peek() == expected {
            self.bump();
            Ok(())
        } else {
            Err(CalcError::Parse(format!(
                "expected {expected:?}, found {:?}",
                self.peek()
            )))
        }
    }

    /// Parse the full input and verify nothing is left unconsumed.
    ///
    /// # Errors
    ///
    /// Returns [`CalcError::Parse`] on any grammatical violation.
    pub fn parse(mut self) -> Result<Expr> {
        let expr = self.expression()?;
        if self.peek() != &Token::Eof {
            return Err(CalcError::Parse(format!(
                "unexpected token after expression: {:?}",
                self.peek()
            )));
        }
        Ok(expr)
    }

    /// Parse an expression, handling `ident = <rhs>` assignment at the top
    /// level before delegating to the Pratt loop.
    fn expression(&mut self) -> Result<Expr> {
        // One-token lookahead: `ident =` signals assignment.
        if let Token::Ident(name) = self.peek().clone() {
            if self.peek_ahead(1) == &Token::Eq {
                self.bump(); // consume ident
                self.bump(); // consume =
                let rhs = self.expression()?; // right-associative
                return Ok(Expr::Assign(name, Box::new(rhs)));
            }
        }
        self.pratt(0)
    }

    /// Pratt loop: parse with minimum left-binding-power `min_bp`.
    fn pratt(&mut self, min_bp: u8) -> Result<Expr> {
        let mut lhs = self.nud()?;

        loop {
            let Some(op) = infix_op(self.peek()) else {
                break;
            };
            let (lbp, rbp) = infix_bp(op);
            if lbp < min_bp {
                break;
            }
            self.bump(); // consume operator
            let rhs = self.pratt(rbp)?;
            lhs = Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    /// Null denotation: parse a token in prefix position.
    fn nud(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.bump();
                Ok(Expr::Number(n))
            }

            Token::Ident(name) => {
                self.bump();
                if self.peek() == &Token::LParen {
                    self.bump(); // consume '('
                    self.parse_call(name)
                } else {
                    Ok(Expr::Var(name))
                }
            }

            Token::Minus => {
                self.bump();
                let operand = self.pratt(PREFIX_BP)?;
                Ok(Expr::Neg(Box::new(operand)))
            }

            // Unary `+` — identity, useful for `+inf`
            Token::Plus => {
                self.bump();
                self.pratt(PREFIX_BP)
            }

            Token::LParen => {
                self.bump();
                let inner = self.expression()?;
                self.expect(&Token::RParen)?;
                Ok(inner)
            }

            other => Err(CalcError::Parse(format!(
                "unexpected token in expression: {other:?}"
            ))),
        }
    }

    /// Parse a comma-separated argument list; opening `(` has already been
    /// consumed.
    fn parse_call(&mut self, name: String) -> Result<Expr> {
        let mut args = Vec::new();

        if self.peek() != &Token::RParen {
            args.push(self.pratt(0)?);
            while self.peek() == &Token::Comma {
                self.bump();
                args.push(self.pratt(0)?);
            }
        }

        self.expect(&Token::RParen)?;
        Ok(Expr::Call { name, args })
    }
}

/// Binding power for prefix `-` / `+`.
const PREFIX_BP: u8 = 9;

/// Map a token to its infix [`BinOp`], if applicable.
#[inline]
fn infix_op(tok: &Token) -> Option<BinOp> {
    match tok {
        Token::Plus => Some(BinOp::Add),
        Token::Minus => Some(BinOp::Sub),
        Token::Star => Some(BinOp::Mul),
        Token::Slash => Some(BinOp::Div),
        Token::Percent => Some(BinOp::Rem),
        Token::Caret => Some(BinOp::Pow),
        _ => None,
    }
}

/// `(left_bp, right_bp)` for each binary operator.
///
/// Right-associative operators have `right_bp < left_bp`.
#[inline]
const fn infix_bp(op: BinOp) -> (u8, u8) {
    match op {
        BinOp::Add | BinOp::Sub => (1, 2),
        BinOp::Mul | BinOp::Div | BinOp::Rem => (3, 4),
        BinOp::Pow => (6, 5), // right-assoc: 2^3^2 = 2^(3^2) = 512
    }
}
