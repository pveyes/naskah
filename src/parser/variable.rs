use super::expr::variable_value;
use super::identifier::identifier;
#[cfg(test)]
use ast::Identifier;
#[cfg(test)]
use ast::Literal;
use ast::VariableDeclaration;
#[cfg(test)]
use ast::VariableValue;

named!(
  pub variable<VariableDeclaration>,
  do_parse!(
    tag!("misal")
      >> id: ws!(identifier)
      >> tag!("=")
      >> expr: variable_value
      >> tag!(";")
      >> (VariableDeclaration {
        id: id,
        value: expr
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
