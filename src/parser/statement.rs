use super::variable::variable_declaration;
use ast::BlockStatement;
use ast::Statement;

named!(
    statement<Statement>,
    alt_complete!(
        map!(variable_declaration, |v| Statement::VariableDeclaration(v))
            | map!(block_statement, |b| Statement::BlockStatement(b))
            | loop_statement
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
    do_parse!(ws!(tag!("loop")) >> s: block_statement >> (Statement::Loop(s)))
);

#[cfg(test)]
mod test {
    use super::*;
    use ast::*;

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
            statement(&b"loop {\nmisal y = kosong;\n}"[..]),
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
}
