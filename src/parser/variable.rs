use super::identifier::identifier;
use super::literal::literal;
#[cfg(test)]
use ast::Identifier;
#[cfg(test)]
use ast::LiteralValue;
use ast::VariableDeclaration;

named!(
  pub variable<VariableDeclaration>,
  do_parse!(
    tag!("misal")
      >> id: map_res!(ws!(take_until!("=")), identifier)
      >> tag!("=")
      >> expr: map_res!(ws!(take_until!(";")), literal)
      >> tag!(";")
      >> (VariableDeclaration {
        id: id.1,
        value: expr.1
      })
  )
);

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn boolean_assignment() {
    assert_eq!(
      variable(&b"misal x = benar;"[..]),
      Ok((
        &b""[..],
        VariableDeclaration {
          id: Identifier {
            name: String::from("x")
          },
          value: LiteralValue::Boolean(true),
        }
      ))
    );
  }

  #[test]
  fn string_assignment() {
    assert_eq!(
      variable(&b"misal x = \"str\";"[..]),
      Ok((
        &b""[..],
        VariableDeclaration {
          id: Identifier {
            name: String::from("x")
          },
          value: LiteralValue::String(String::from("str")),
        }
      ))
    );
  }

  #[test]
  fn number_assignment() {
    assert_eq!(
      // TODO fix space at the end bug
      variable(&b"misal x = 5 ;"[..]),
      Ok((
        &b""[..],
        VariableDeclaration {
          id: Identifier {
            name: String::from("x")
          },
          value: LiteralValue::Number(5),
        }
      ))
    );
  }

  #[test]
  fn null_assignment() {
    assert_eq!(
      variable(&b"misal x = kosong;"[..]),
      Ok((
        &b""[..],
        VariableDeclaration {
          id: Identifier {
            name: String::from("x")
          },
          value: LiteralValue::Null,
        }
      ))
    );
  }
}
