pub mod ast;
mod lexer;
mod syntax;

pub use self::lexer::ParseError;

use self::ast::Program;

/// Read a program. Only the writing is checked, not whether the names exist.
pub fn parse(input: &str) -> Result<Program, ParseError> {
    syntax::parse_program(input)
}

/// Read a program and also check that every name it uses has been made, suggesting
/// the nearest name when one looks like a typo.
pub fn parse_checked(input: &str) -> Result<Program, ParseError> {
    syntax::parse_checked(input)
}
