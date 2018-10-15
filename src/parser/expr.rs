use super::identifier::identifier;
use super::literal::literal;
use ast::BinaryExpression;
use ast::Operator;
use ast::VariableValue;

named!(
  pub variable_value<VariableValue>,
  ws!(alt_complete!(
    map!(literal, |l| VariableValue::Literal(l))
      | map!(identifier, |i| VariableValue::Identifier(i))
  ))
);

named!(
  operator<Operator>,
  alt_complete!(
    map!(tag!("+"), |_| Operator::Plus)
      | map!(tag!("-"), |_| Operator::Minus)
      | map!(tag!("=="), |_| Operator::Equal)
      | map!(tag!("!="), |_| Operator::NotEqual)
      | map!(tag!(">"), |_| Operator::GreaterThan)
      | map!(tag!("<"), |_| Operator::LessThan)
  )
);

named!(
  binary_expression<BinaryExpression>,
  do_parse!(
    v1: variable_value
      >> op: operator
      >> v2: ws!(variable_value)
      >> tag!(";")
      >> (BinaryExpression {
        left: v1,
        right: v2,
        operator: op,
      })
  )
);

#[cfg(test)]
mod test {
  use super::*;
  use ast::Identifier;
  use ast::Literal;

  #[test]
  fn var() {
    assert_eq!(
      variable_value(&b" kosong;"[..]),
      Ok((&b";"[..], VariableValue::Literal(Literal::Null)))
    );
  }

  #[test]
  fn op() {
    assert_eq!(operator(&b"+"[..]), Ok((&b""[..], Operator::Plus)));
    assert_eq!(operator(&b"-"[..]), Ok((&b""[..], Operator::Minus)));
    assert_eq!(operator(&b"=="[..]), Ok((&b""[..], Operator::Equal)));
    assert_eq!(operator(&b"!="[..]), Ok((&b""[..], Operator::NotEqual)));
    assert_eq!(operator(&b">"[..]), Ok((&b""[..], Operator::GreaterThan)));
    assert_eq!(operator(&b"<"[..]), Ok((&b""[..], Operator::LessThan)));
  }

  #[test]
  fn binary_expression_literals() {
    assert_eq!(
      binary_expression(&b"\"kosong\" != kosong;"[..]),
      Ok((
        &b""[..],
        BinaryExpression {
          left: VariableValue::Literal(Literal::String(String::from("kosong"))),
          right: VariableValue::Literal(Literal::Null),
          operator: Operator::NotEqual
        }
      ))
    )
  }

  #[test]
  fn binary_expression_literal_id() {
    assert_eq!(
      binary_expression(&b"x != 5;"[..]),
      Ok((
        &b""[..],
        BinaryExpression {
          left: VariableValue::Identifier(Identifier {
            name: String::from("x")
          }),
          right: VariableValue::Literal(Literal::Number(5)),
          operator: Operator::GreaterThan
        }
      ))
    )
  }
}
