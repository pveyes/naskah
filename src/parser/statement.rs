use super::expr::binary_expression;
use super::identifier::identifier;
use super::literal::{boolean_literal, null_literal};
use super::variable::variable_declaration;
use ast::*;

named!(
    statement<Statement>,
    alt_complete!(
        if_else_statement
            | if_statement
            | loop_statement
            | map!(block_statement, |b| Statement::BlockStatement(b))
            | map!(variable_declaration, |v| Statement::VariableDeclaration(v))
    )
);

named!(
    block_statement<BlockStatement>,
    map!(
        // block statement can contains nothing, that's why we need to
        // exclude any whitespace
        delimited!(tag!("{"), ws!(opt!(many1!(statement))), tag!("}")),
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
    if_statement<Statement>,
    preceded!(
        tag!("jika "),
        do_parse!(
            expr: alt_complete!(special_if_condition | binary_expression)
                >> tag!(" ")
                >> st: block_statement
                >> (Statement::IfStatement(IfStatement {
                    test: expr,
                    consequent: st,
                    alternate: None,
                }))
        )
    )
);

named!(
    if_else_statement<Statement>,
    preceded!(
        tag!("jika "),
        do_parse!(
            expr: alt_complete!(special_if_condition | binary_expression)
                >> tag!(" ")
                >> st: block_statement
                >> els: else_statement
                >> (Statement::IfStatement(IfStatement {
                    test: expr,
                    consequent: st,
                    alternate: els,
                }))
        )
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_block_statement() {
        assert_eq!(
            statement(&b"{\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(BlockStatement { body: None })
            ))
        );
    }

    #[test]
    fn var_decl_inside_block() {
        assert_eq!(
            statement(&b"{\nmisal x = 5;\n}"[..]),
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
            statement(&b"{\n{\n}\n}"[..]),
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
            statement(&b"ulang {\nmisal y = kosong;\n}"[..]),
            Ok((
                &b""[..],
                Statement::Loop(BlockStatement {
                    body: Some(vec![Statement::VariableDeclaration(VariableDeclaration {
                        id: Identifier {
                            name: String::from("y")
                        },
                        value: Expression::Literal(Literal::Null)
                    })])
                })
            ))
        );
    }

    #[test]
    fn test_if_statement() {
        assert_eq!(
            statement(&b"jika a == salah {\nmisal z = benar;\n}"[..]),
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
        use std::str;
        println!(
            "pret xx{}",
            str::from_utf8(&vec![32, 97, 116, 97, 117, 32, 123, 10, 125]).unwrap()
        );
        assert_eq!(
            statement(&b"jika a == salah {\n} atau {\n}"[..]),
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
            statement(&b"jika a benar {\n}"[..]),
            statement(&b"jika a == benar {\n}"[..])
        );

        assert_eq!(
            statement(&b"jika a salah {\n} atau {\n}"[..]),
            statement(&b"jika a == salah {\n} atau {\n}"[..]),
        );

        assert_eq!(
            statement(&b"jika a kosong {\n}"[..]),
            statement(&b"jika a == kosong {\n}"[..])
        );
    }
}
