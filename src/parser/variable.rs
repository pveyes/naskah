use super::identifier::identifier;
use super::literal::literal;
#[cfg(test)]
use ast::Identifier;
#[cfg(test)]
use ast::Literal;
use ast::VariableDeclaration;
use ast::VariableValue;

named!(
  variable_value<VariableValue>,
  alt_complete!(
    map!(literal, |l| VariableValue::Literal(l))
      | map!(identifier, |i| VariableValue::Identifier(i))
  )
);

named!(
  pub variable<VariableDeclaration>,
  do_parse!(
    tag!("misal")
      >> id: map_res!(ws!(take_until!("=")), identifier)
      >> tag!("=")
      >> expr: map_res!(ws!(take_until!(";")), variable_value)
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
          value: VariableValue::Literal(Literal::Boolean(true)),
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
          value: VariableValue::Literal(Literal::String(String::from("str"))),
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
          value: VariableValue::Literal(Literal::Number(5)),
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
          value: VariableValue::Literal(Literal::Null),
        }
      ))
    );
  }

  #[test]
  fn identifier_assignment() {
    assert_eq!(
      variable(&b"misal x = a;"[..]),
      Ok((
        &b""[..],
        VariableDeclaration {
          id: Identifier {
            name: String::from("x")
          },
          value: VariableValue::Identifier(Identifier {
            name: String::from("a")
          }),
        }
      ))
    );
  }
}
