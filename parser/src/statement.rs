use super::ast::*;
use super::expr::{binary_expression, expression};
use super::identifier::identifier;
use super::literal::{boolean_literal, null_literal};
use super::variable::variable_declaration;

named!(
    break_statement<Statement>,
    map!(tag!("berhenti;"), |_| Statement::Break)
);

named!(
    continue_statement<Statement>,
    map!(tag!("lanjut;"), |_| Statement::Continue)
);

named!(
    expression_statement<Statement>,
    map!(do_parse!(e: expression >> tag!(";") >> (e)), |e| {
        Statement::Expression(e)
    })
);

named!(
    pub parse_statement<Statement>,
    alt_complete!(
            loop_statement
            // TODO unsyntactic break/continue
            | break_statement
            | continue_statement
            | map!(if_statement, |b| Statement::IfStatement(b))
            | map!(block_statement, |b| Statement::BlockStatement(b))
            | map!(variable_declaration, |v| Statement::VariableDeclaration(v))
            | expression_statement
    )
);

named!(
    block_statement<BlockStatement>,
    map!(
        // block statement can contains nothing, that's why we need to
        // exclude any whitespace
        delimited!(
            tag!("{"),
            ws!(opt!(many1!(ws!(parse_statement)))),
            tag!("}")
        ),
        |body| BlockStatement { body }
    )
);

named!(
    loop_statement<Statement>,
    map!(
        // either ulang{} or ulang { }
        preceded!(ws!(tag!("ulang")), block_statement),
        Statement::Loop
    )
);

named!(
    special_if_condition<Expression>,
    do_parse!(
        left: identifier
            >> tag!(" ")
            >> right: alt_complete!(null_literal | boolean_literal)
            >> (Expression::BinaryExpression(Box::new(BinaryExpression {
                left: Expression::Identifier(left),
                right: Expression::Literal(right),
                operator: Operator::Equal
            })))
    )
);

named!(
    else_statement<Option<AlternateStatement>>,
    opt!(preceded!(
        tag!(" atau "),
        alt_complete!(
            map!(if_statement, |s| AlternateStatement::IfStatement(Box::new(
                s
            ))) | map!(block_statement, |s| AlternateStatement::BlockStatement(s))
        )
    ))
);

named!(
    single_if_statement<IfStatement>,
    preceded!(
        tag!("jika "),
        do_parse!(
            expr: alt_complete!(special_if_condition | binary_expression)
                >> tag!(" ")
                >> st: block_statement
                >> (IfStatement {
                    test: expr,
                    consequent: st,
                    alternate: None,
                })
        )
    )
);

named!(
    if_else_statement<IfStatement>,
    preceded!(
        tag!("jika "),
        do_parse!(
            expr: alt_complete!(special_if_condition | binary_expression)
                >> tag!(" ")
                >> st: block_statement
                >> els: else_statement
                >> (IfStatement {
                    test: expr,
                    consequent: st,
                    alternate: els,
                })
        )
    )
);

named!(
    if_statement<IfStatement>,
    // the reason we put if_else_statement first is because the
    // similarities of both syntax, considering single_if_statement will
    // also parse if_else_statement (albeit returning Incomplete), we
    // have to prioritize if_else and only parse single_if_statement
    // if the first parser failed. maybe not the most performant, but it works
    alt_complete!(if_else_statement | single_if_statement)
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_block_statement() {
        assert_eq!(
            parse_statement(&b"{\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(BlockStatement { body: None })
            ))
        );
    }

    #[test]
    fn var_decl_inside_block() {
        assert_eq!(
            parse_statement(&b"{\nmisal x = 5;\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(BlockStatement {
                    body: Some(vec![Statement::VariableDeclaration(VariableDeclaration {
                        id: Identifier {
                            name: String::from("x")
                        },
                        value: Expression::Literal(Literal::Number(5))
                    })])
                })
            ))
        );
    }

    #[test]
    fn recursive_empty_block_statement() {
        assert_eq!(
            parse_statement(&b"{\n{\n}\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(BlockStatement {
                    body: Some(vec![Statement::BlockStatement(BlockStatement {
                        body: None
                    })])
                })
            ))
        );
    }

    #[test]
    fn test_loop_statement() {
        assert_eq!(
            parse_statement(&b"ulang {\nberhenti;\nlanjut;\n}"[..]),
            Ok((
                &b""[..],
                Statement::Loop(BlockStatement {
                    body: Some(vec![Statement::Break, Statement::Continue])
                })
            ))
        );
    }

    #[test]
    fn test_if_statement() {
        assert_eq!(
            parse_statement(&b"jika a == salah {\nmisal z = benar;\n}"[..]),
            Ok((
                &b""[..],
                Statement::IfStatement(IfStatement {
                    test: Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: Expression::Identifier(Identifier {
                            name: String::from("a")
                        }),
                        right: Expression::Literal(Literal::Boolean(false)),
                        operator: Operator::Equal
                    })),
                    consequent: BlockStatement {
                        body: Some(vec![Statement::VariableDeclaration(VariableDeclaration {
                            id: Identifier {
                                name: String::from("z")
                            },
                            value: Expression::Literal(Literal::Boolean(true))
                        })])
                    },
                    alternate: None
                })
            ))
        );
    }

    #[test]
    fn test_if_else_statement() {
        assert_eq!(
            parse_statement(&b"jika a == salah {\n} atau {\n}"[..]),
            Ok((
                &b""[..],
                Statement::IfStatement(IfStatement {
                    test: Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: Expression::Identifier(Identifier {
                            name: String::from("a")
                        }),
                        right: Expression::Literal(Literal::Boolean(false)),
                        operator: Operator::Equal
                    })),
                    consequent: BlockStatement { body: None },
                    alternate: Some(AlternateStatement::BlockStatement(BlockStatement {
                        body: None
                    }))
                })
            ))
        );
    }

    #[test]
    fn if_special() {
        assert_eq!(
            parse_statement(&b"jika a benar {\n}"[..]),
            parse_statement(&b"jika a == benar {\n}"[..])
        );

        assert_eq!(
            parse_statement(&b"jika a salah {\n} atau {\n}"[..]),
            parse_statement(&b"jika a == salah {\n} atau {\n}"[..]),
        );

        assert_eq!(
            parse_statement(&b"jika a kosong {\n}"[..]),
            parse_statement(&b"jika a == kosong {\n}"[..])
        );
    }

    #[test]
    fn recursive_if_else() {
        assert_eq!(
            parse_statement(&b"jika a benar {\n} atau jika b kosong {\n}"[..]),
            Ok((
                &b""[..],
                Statement::IfStatement(IfStatement {
                    test: Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: Expression::Identifier(Identifier {
                            name: String::from("a")
                        }),
                        right: Expression::Literal(Literal::Boolean(true)),
                        operator: Operator::Equal
                    })),
                    consequent: BlockStatement { body: None },
                    alternate: Some(AlternateStatement::IfStatement(Box::new(IfStatement {
                        test: Expression::BinaryExpression(Box::new(BinaryExpression {
                            left: Expression::Identifier(Identifier {
                                name: String::from("b")
                            }),
                            right: Expression::Literal(Literal::Null),
                            operator: Operator::Equal
                        })),
                        consequent: BlockStatement { body: None },
                        alternate: None
                    })))
                })
            ))
        );

        assert_eq!(
            parse_statement(&b"jika c == 2 {\n} atau jika d benar {\n} atau {\n}"[..]),
            Ok((
                &b""[..],
                Statement::IfStatement(IfStatement {
                    test: Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: Expression::Identifier(Identifier {
                            name: String::from("c")
                        }),
                        right: Expression::Literal(Literal::Number(2)),
                        operator: Operator::Equal
                    })),
                    consequent: BlockStatement { body: None },
                    alternate: Some(AlternateStatement::IfStatement(Box::new(IfStatement {
                        test: Expression::BinaryExpression(Box::new(BinaryExpression {
                            left: Expression::Identifier(Identifier {
                                name: String::from("d")
                            }),
                            right: Expression::Literal(Literal::Boolean(true)),
                            operator: Operator::Equal
                        })),
                        consequent: BlockStatement { body: None },
                        alternate: Some(AlternateStatement::BlockStatement(BlockStatement {
                            body: None
                        }))
                    })))
                })
            ))
        );
    }

    #[test]
    fn simple_expression_statement() {
        assert_eq!(
            parse_statement(&b"alert();"[..]),
            Ok((
                &b""[..],
                Statement::Expression(Expression::CallExpression(CallExpression {
                    callee: Identifier {
                        name: String::from("alert")
                    },
                    arguments: vec![]
                }))
            ))
        );
    }

    #[test]
    fn reassignment() {
        assert_eq!(
            parse_statement(&b"x = x ^ 5;"[..]),
            Ok((
                &b""[..],
                Statement::Expression(Expression::Assignment(AssignmentExpression {
                    id: Identifier {
                        name: String::from("x"),
                    },
                    value: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
                        left: Expression::Identifier(Identifier {
                            name: String::from("x"),
                        }),
                        right: Expression::Literal(Literal::Number(5)),
                        operator: Operator::Exponentiation,
                    })))
                }))
            ))
        );
    }
}
