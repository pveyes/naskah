use super::ast::*;
use super::identifier::identifier;
use super::literal::literal;

named!(
    assignment_expression<Expression>,
    do_parse!(
        i: identifier
            >> ws!(tag!("="))
            >> e: expression
            >> (Expression::Assignment(AssignmentExpression {
                id: i,
                value: Box::new(e)
            }))
    )
);

named!(
    fn_arguments<Vec<Expression>>,
    // this is buggy because we can do fn(a b)
    // TODO fix this
    fold_many0!(
        do_parse!(e: expression >> opt!(tag!(",")) >> (e)),
        Vec::new(),
        |mut acc: Vec<Expression>, item| {
            acc.push(item);
            acc
        }
    )
);

named!(
    call_expression<Expression>,
    do_parse!(
        c: identifier
            >> tag!("(")
            >> args: fn_arguments
            >> tag!(")")
            >> (Expression::CallExpression(CallExpression {
                callee: c,
                arguments: args
            }))
    )
);

named!(
    simple_expression<Expression>,
    alt_complete!(
        map!(literal, |l| Expression::Literal(l)) | map!(identifier, |i| Expression::Identifier(i))
    )
);

named!(
  pub expression<Expression>,
  alt_complete!(
      assignment_expression |
      call_expression |
      binary_expression |
      simple_expression
  )
);

named!(
    operator<Operator>,
    // we use alt_complete here just to be safe
    // because operator can contains different length
    alt_complete!(
        map!(tag!("+"), |_| Operator::Addition)
            | map!(tag!("-"), |_| Operator::Substraction)
            | map!(tag!("*"), |_| Operator::Multiplication)
            | map!(tag!("/"), |_| Operator::Division)
            | map!(tag!("%"), |_| Operator::Remainder)
            | map!(tag!("^"), |_| Operator::Exponentiation)
            | map!(tag!("=="), |_| Operator::Equal)
            | map!(tag!("!="), |_| Operator::NotEqual)
            | map!(tag!(">="), |_| Operator::GreaterThanOrEqualTo)
            | map!(tag!("<="), |_| Operator::LessThanOrEqualTo)
            | map!(tag!(">"), |_| Operator::GreaterThan)
            | map!(tag!("<"), |_| Operator::LessThan)
    )
);

named!(
    pub binary_expression<Expression>,
    do_parse!(
        first: simple_expression
            >> fold: fold_many0!(
                do_parse!(op: ws!(operator) >> expr: simple_expression >> (op, expr)),
                first,
                |left: Expression, (operator, right): (Operator, Expression)| {
                    Expression::BinaryExpression(Box::new(BinaryExpression {
                        left,
                        right,
                        operator,
                    }))
                }
            )
            >> (fold)
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn op() {
        assert_eq!(operator(&b"+"[..]), Ok((&b""[..], Operator::Addition)));
        assert_eq!(operator(&b"-"[..]), Ok((&b""[..], Operator::Substraction)));
        assert_eq!(
            operator(&b"*"[..]),
            Ok((&b""[..], Operator::Multiplication))
        );
        assert_eq!(operator(&b"/"[..]), Ok((&b""[..], Operator::Division)));
        assert_eq!(operator(&b"%"[..]), Ok((&b""[..], Operator::Remainder)));
        assert_eq!(
            operator(&b"^"[..]),
            Ok((&b""[..], Operator::Exponentiation))
        );
        assert_eq!(operator(&b"=="[..]), Ok((&b""[..], Operator::Equal)));
        assert_eq!(operator(&b"!="[..]), Ok((&b""[..], Operator::NotEqual)));
        assert_eq!(operator(&b">"[..]), Ok((&b""[..], Operator::GreaterThan)));
        assert_eq!(operator(&b"<"[..]), Ok((&b""[..], Operator::LessThan)));
        assert_eq!(operator(&b">="[..]), Ok((&b""[..], Operator::GreaterThanOrEqualTo)));
        assert_eq!(operator(&b"<="[..]), Ok((&b""[..], Operator::LessThanOrEqualTo)));
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
                    operator: Operator::Addition,
                }))
            ))
        );
    }

    #[test]
    fn basic_call_expression() {
        assert_eq!(
            expression(&b"hello()"[..]),
            Ok((
                &b""[..],
                Expression::CallExpression(CallExpression {
                    callee: Identifier {
                        name: String::from("hello")
                    },
                    arguments: vec![]
                })
            ))
        );
    }

    #[test]
    fn fn_with_arguments() {
        assert_eq!(
            expression(&b"tulis(\"hello, world!\")"[..]),
            Ok((
                &b""[..],
                Expression::CallExpression(CallExpression {
                    callee: Identifier {
                        name: String::from("tulis")
                    },
                    arguments: vec![Expression::Literal(Literal::String(String::from(
                        "hello, world!"
                    )))]
                })
            ))
        );
    }
}
