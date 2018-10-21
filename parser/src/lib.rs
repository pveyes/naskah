#[macro_use]
extern crate nom;
extern crate regex;

pub mod ast;
mod expr;
mod identifier;
mod literal;
mod number;
mod statement;
mod variable;

use self::ast::*;
use self::statement::parse_statement;

named!(
  pub program<Program>,
  map!(
    many0!(
      do_parse!(s: parse_statement >> tag!("\n") >> (s))
    ),
    |body| Program { body }
  )
);

pub fn parse(input: &str) -> Result<Program, nom::Err<&[u8]>> {
  let (_res, p) = program(input.as_bytes())?;
  Ok(p)
}
