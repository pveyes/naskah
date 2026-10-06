pub mod ast;
mod lexer;
mod syntax;

pub use self::lexer::ParseError;

use self::ast::Program;

pub fn parse(input: &str) -> Result<Program, ParseError> {
    syntax::parse_program(input)
}
