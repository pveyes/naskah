use super::expr::expression;
use super::identifier::identifier;
use ast::VariableDeclaration;

named!(
  pub variable<VariableDeclaration>,
  do_parse!(
    tag!("misal")
      >> id: ws!(identifier)
      >> tag!("=")
      >> expr: ws!(expression)
      >> (VariableDeclaration {
        id: id,
        value: expr
      })
  )
);

#[cfg(test)]
mod test {
    use super::*;
    use ast::BinaryExpression;
    use ast::Expression;
    use ast::Identifier;
    use ast::Literal;
    use ast::Operator;

    #[test]
    fn boolean_assignment() {
        assert_eq!(
            variable(&b"misal x = benar;z"[..]),
            Ok((
                &b"z"[..],
                VariableDeclaration {
                    id: Identifier {
                        name: String::from("x")
                    },
                    value: Expression::Literal(Literal::Boolean(true)),
                }
            ))
        );
    }

    #[test]
    fn string_assignment() {
        assert_eq!(
            variable(&b"misal x = \"str\";rest"[..]),
            Ok((
                &b"rest"[..],
                VariableDeclaration {
                    id: Identifier {
                        name: String::from("x")
                    },
                    value: Expression::Literal(Literal::String(String::from("str"))),
                }
            ))
        );
    }

    #[test]
    fn number_assignment() {
        assert_eq!(
            variable(&b"misal x = 5;rest"[..]),
            Ok((
                &b"rest"[..],
                VariableDeclaration {
                    id: Identifier {
                        name: String::from("x")
                    },
                    value: Expression::Literal(Literal::Number(5)),
                }
            ))
        );
    }

    #[test]
    fn null_assignment() {
        assert_eq!(
            variable(&b"misal x = kosong;rest"[..]),
            Ok((
                &b"rest"[..],
                VariableDeclaration {
                    id: Identifier {
                        name: String::from("x")
                    },
                    value: Expression::Literal(Literal::Null),
                }
            ))
        );
    }

    #[test]
    fn identifier_assignment() {
        assert_eq!(
            variable(&b"misal x = a;rest"[..]),
            Ok((
                &b"rest"[..],
                VariableDeclaration {
                    id: Identifier {
                        name: String::from("x")
                    },
                    value: Expression::Identifier(Identifier {
                        name: String::from("a")
                    }),
                }
            ))
        );
    }

    #[test]
    fn binary_expression_assignment() {
        assert_eq!(
            variable(&b"misal sum = 2 + 3;rest"[..]),
            Ok((
                &b"rest"[..],
                VariableDeclaration {
                    id: Identifier {
                        name: String::from("sum")
                    },
                    value: Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: Expression::Literal(Literal::Number(2)),
                        right: Expression::Literal(Literal::Number(3)),
                        operator: Operator::Plus,
                    }))
                }
            ))
        );
    }
}
