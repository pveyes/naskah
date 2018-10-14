use super::boolean::boolean_literal;
use super::identifier::identifier;
use super::number::number_literal;
#[cfg(test)]
use ast::Identifier;
use ast::LiteralValue;
use ast::VariableDeclaration;

named!(
  literal<LiteralValue>,
  alt_complete!(boolean_literal | number_literal)
);

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
  fn test_literal() {
    assert_eq!(
      literal(&b"benar"[..]),
      Ok((&b""[..], LiteralValue::Boolean(true)))
    );

    assert_eq!(
      literal(&b"salah"[..]),
      Ok((&b""[..], LiteralValue::Boolean(false)))
    );

    assert_eq!(
      literal(&b"2 "[..]),
      Ok((&b" "[..], LiteralValue::Number(2)))
    );
  }

  #[test]
  fn test_parse_variable() {
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
}
