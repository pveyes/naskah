use ast::Identifier;
use ast::LiteralValue;
use ast::VariableDeclaration;
use parser::boolean::boolean_literal;
use parser::number::number_literal;
use std::str;

named!(
  literal<LiteralValue>,
  alt_complete!(number_literal | boolean_literal)
);

#[test]
fn test_literal() {
  let (_, result) = literal(&b"benar;"[..]).unwrap();
  assert_eq!(result, LiteralValue::Boolean(true));

  let (_, result) = literal(&b"salah;"[..]).unwrap();
  assert_eq!(result, LiteralValue::Boolean(false));

  let (_, result) = literal(&b"2;"[..]).unwrap();
  assert_eq!(result, LiteralValue::Number(2));
}

named!(
  variable<VariableDeclaration>,
  do_parse!(
    tag!("misal")
      >> name: map_res!(take_until!("="), str::from_utf8)
      >> tag!("=")
      >> value: map!(take_until!(";"), literal)
      >> tag!(";")
      >> (VariableDeclaration {
        id: Identifier {
          name: String::from(name.trim()),
        },
        value: value,
      })
  )
);

#[test]
fn test_parse_variable() {
  let (_, result) = variable(&b"misal x = benar;"[..]).unwrap();

  assert_eq!(
    result,
    VariableDeclaration {
      id: Identifier {
        name: String::from("x")
      },
      value: LiteralValue::Boolean(true),
    }
  );

  let (_, result) = variable(&b"misal x = 5;"[..]).unwrap();

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
