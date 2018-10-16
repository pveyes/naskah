use super::identifier::identifier;
use super::literal::literal;
use ast::BinaryExpression;
use ast::Expression;
use ast::Operator;

named!(
    simple_expression<Expression>,
    ws!(alt_complete!(
        map!(literal, |l| Expression::Literal(l)) | map!(identifier, |i| Expression::Identifier(i))
    ))
);

named!(
  pub expression<Expression>,
  do_parse!(
    expr: alt_complete!(
        binary_expression |
        simple_expression
    ) >> (expr)
  )
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
    binary_expression<Expression>,
    do_parse!(
        first: simple_expression
            >> fold: fold_many0!(
                do_parse!(op: operator >> expr: simple_expression >> (op, expr)),
                first,
                |expr1: Expression, (op, expr2): (Operator, Expression)| {
                    Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: expr1,
                        right: expr2,
                        operator: op,
                    }))
                }
            )
            >> (fold)
    )
);

#[cfg(test)]
mod test {
    use super::*;
    use ast::Identifier;
    use ast::Literal;

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
            expression(&b"\"kosong\" != kosong;"[..]),
            Ok((
                &b";"[..],
                Expression::BinaryExpression(Box::new(BinaryExpression {
                    left: Expression::Literal(Literal::String(String::from("kosong"))),
                    right: Expression::Literal(Literal::Null),
                    operator: Operator::NotEqual
                }))
            ))
        )
    }

    #[test]
    fn binary_expression_as_expression() {
        assert_eq!(
            expression(&b"x > 5;"[..]),
            Ok((
                &b";"[..],
                Expression::BinaryExpression(Box::new(BinaryExpression {
                    left: Expression::Identifier(Identifier {
                        name: String::from("x")
                    }),
                    right: Expression::Literal(Literal::Number(5)),
                    operator: Operator::GreaterThan
                }))
            ))
        )
    }

    #[test]
    fn recursive_binary_expression() {
        assert_eq!(
            expression(&b"1 > 2 + 3;"[..]),
            Ok((
                &b";"[..],
                Expression::BinaryExpression(Box::new(BinaryExpression {
                    left: Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: Expression::Literal(Literal::Number(1)),
                        right: Expression::Literal(Literal::Number(2)),
                        operator: Operator::GreaterThan,
                    })),
                    right: Expression::Literal(Literal::Number(3)),
                    operator: Operator::Plus,
                }))
            ))
        );
    }
}
