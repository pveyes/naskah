mod boolean;
mod identifier;
mod number;
mod variable;

use ast::VariableDeclaration;
use nom;

pub fn parse(input: &str) -> Result<VariableDeclaration, nom::Err<&[u8]>> {
  let (_, decl) = variable::variable(input.as_bytes())?;
  Ok(decl)
}
