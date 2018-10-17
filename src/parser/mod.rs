mod expr;
mod identifier;
mod literal;
mod number;
mod statement;
mod variable;

use self::statement::statement;
use ast::Program;
use nom;

named!(
  pub program<Program>,
  map!(
    many0!(
      do_parse!(s: statement >> tag!("\n") >> (s))
    ),
    |body| Program { body }
  )
);

pub fn parse(input: &str) -> Result<Program, nom::Err<&[u8]>> {
    let (_res, p) = program(input.as_bytes())?;
    Ok(p)
}
