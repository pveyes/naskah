use super::expr::expression;
use super::identifier::identifier;
use ast::VariableDeclaration;

named!(
  pub variable_declaration<VariableDeclaration>,
  preceded!(tag!("misal "), do_parse!(
      id: identifier
      // we can do either x = 2 or x=2
      // both is fine
      >> ws!(tag!("="))
      >> expr: expression
      >> tag!(";")
      >> (VariableDeclaration {
        id: id,
        value: expr
      })

  ))
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
            variable_declaration(&b"misal x = benar;"[..]),
            Ok((
                &b""[..],
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
            variable_declaration(&b"misal x = \"str\";"[..]),
            Ok((
                &b""[..],
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
            variable_declaration(&b"misal x = 5;"[..]),
            Ok((
                &b""[..],
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
            variable_declaration(&b"misal x = kosong;"[..]),
            Ok((
                &b""[..],
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
            variable_declaration(&b"misal x = a;"[..]),
            Ok((
                &b""[..],
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
            variable_declaration(&b"misal sum = 2 + 3;"[..]),
            Ok((
                &b""[..],
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
