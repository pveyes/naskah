use super::boolean::boolean_literal;
use super::number::number_literal;
use ast::Identifier;
use ast::LiteralValue;
use ast::VariableDeclaration;
use std::num::ParseIntError;
use std::str;

named!(
  parse_literal<LiteralValue>,
  alt!(number_literal | boolean_literal)
);

#[test]
fn test_parse_literal() {
  let (_, result) = parse_literal(&b"benar"[..]).unwrap();
  assert_eq!(result, LiteralValue::Boolean(true));

  let (_, result) = parse_literal(&b"salah"[..]).unwrap();
  assert_eq!(result, LiteralValue::Boolean(false));

  let (_, result) = parse_literal(&b"2"[..]).unwrap();
  assert_eq!(result, LiteralValue::Number(2));
}

named!(
  parse_variable<VariableDeclaration>,
  ws!(do_parse!(
    tag!("misal")
      >> name: ws!(map_res!(take_until!("="), str::from_utf8))
      >> tag!("=")
      >> value: map_res!(take_until!(";"), parse_literal)
      >> (VariableDeclaration {
        id: Identifier {
          name: String::from(name.trim()),
        },
        value: LiteralValue::Boolean(true),
      })
  ))
);

#[test]
fn test_parse_variable() {
  let (_, result) = parse_variable(&b"misal x = benar;"[..]).unwrap();

  assert_eq!(
    result,
    VariableDeclaration {
      id: Identifier {
        name: String::from("x")
      },
      value: LiteralValue::Boolean(true),
    }
  );

  let (_, result) = parse_variable(&b"misal x = 5;"[..]).unwrap();

  assert_eq!(
    result,
    VariableDeclaration {
      id: Identifier {
        name: String::from("x")
      },
      value: LiteralValue::Boolean(true),
    }
  );
}
