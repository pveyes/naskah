mod expr;
mod identifier;
mod literal;
mod number;
mod statement;
mod variable;

use ast::VariableDeclaration;
use nom;

pub fn parse(input: &str) -> Result<VariableDeclaration, nom::Err<&[u8]>> {
    let (_, decl) = variable::variable_declaration(input.as_bytes())?;
    Ok(decl)
}
