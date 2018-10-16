use super::expr::expression;
use super::identifier::identifier;
use super::literal::{boolean_literal, null_literal};
use super::variable::variable_declaration;
use ast::*;

named!(
    statement<Statement>,
    alt_complete!(
        map!(variable_declaration, |v| Statement::VariableDeclaration(v))
            | map!(block_statement, |b| Statement::BlockStatement(b))
            | loop_statement
            | if_statement
            | empty_statement
    )
);

named!(
    empty_statement<Statement>,
    map!(tag!(""), |_| Statement::Empty)
);

named!(
    block_statement<BlockStatement>,
    map!(
        do_parse!(tag!("{") >> s: many1!(ws!(statement)) >> tag!("}") >> (s)),
        |s| BlockStatement { body: s }
    )
);

named!(
    loop_statement<Statement>,
    do_parse!(ws!(tag!("ulang")) >> s: block_statement >> (Statement::Loop(s)))
);

named!(
    special_if_condition<Expression>,
    do_parse!(
        left: identifier
            >> right: ws!(alt!(boolean_literal | null_literal))
            >> (Expression::BinaryExpression(Box::new(BinaryExpression {
                left: Expression::Identifier(left),
                right: Expression::Literal(right),
                operator: Operator::Equal
            })))
    )
);

named!(
    if_statement<Statement>,
    do_parse!(
        ws!(tag!("jika"))
            >> expr: alt!(special_if_condition | expression)
            >> s: block_statement
            >> (Statement::IfStatement(IfStatement {
                test: expr,
                consequent: s,
            }))
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_statement() {
        assert_eq!(empty_statement(&b""[..]), Ok((&b""[..], Statement::Empty)));
    }

    #[test]
    fn empty_block() {
        assert_eq!(
            statement(&b"{\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(BlockStatement {
                    body: vec![Statement::Empty]
                })
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
                    body: vec![Statement::VariableDeclaration(VariableDeclaration {
                        id: Identifier {
                            name: String::from("x")
                        },
                        value: Expression::Literal(Literal::Number(5))
                    })]
                })
            ))
        );
    }

    #[test]
    fn recursive_empty_block() {
        assert_eq!(
            statement(&b"{\n{\n}\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(BlockStatement {
                    body: vec![Statement::BlockStatement(BlockStatement {
                        body: vec![Statement::Empty]
                    })]
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
                    body: vec![Statement::VariableDeclaration(VariableDeclaration {
                        id: Identifier {
                            name: String::from("y")
                        },
                        value: Expression::Literal(Literal::Null)
                    })]
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
                        body: vec![Statement::VariableDeclaration(VariableDeclaration {
                            id: Identifier {
                                name: String::from("z")
                            },
                            value: Expression::Literal(Literal::Boolean(true))
                        })]
                    }
                })
            ))
        );
    }

    #[test]
    fn special_if_statement() {
        assert_eq!(
            statement(&b"jika a kosong {\n}"[..]),
            statement(&b"jika a == kosong {\n}"[..])
        );

        assert_eq!(
            statement(&b"jika a benar {\n}"[..]),
            statement(&b"jika a == benar {\n}"[..])
        );

        assert_eq!(
            statement(&b"jika a salah {\n}"[..]),
            statement(&b"jika a == salah {\n}"[..])
        );
    }
}
