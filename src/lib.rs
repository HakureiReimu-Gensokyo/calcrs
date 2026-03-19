#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;

mod tests;

use error::Result;
use eval::Context;
use lexer::Lexer;
use parser::Parser;

pub fn evaluate(source: &str, ctx: &mut Context) -> Result<f64> {
    let tokens = Lexer::new(source).tokenize()?;
    let ast = Parser::new(tokens).parse()?;
    ctx.eval(&ast)
}
