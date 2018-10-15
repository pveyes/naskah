use super::variable::variable_declaration;
use ast::Statement;

named!(
    statement<Statement>,
    alt_complete!(
        map!(variable_declaration, |v| Statement::VariableDeclaration(v))
            | block_statement
            | empty_statement
    )
);

named!(
    empty_statement<Statement>,
    map!(tag!(""), |_| Statement::Empty)
);

named!(
    block_statement<Statement>,
    map!(
        do_parse!(tag!("{") >> s: many1!(ws!(statement)) >> tag!("}") >> (s)),
        Statement::BlockStatement
    )
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
            block_statement(&b"{\n}"[..]),
            Ok((&b""[..], Statement::BlockStatement(vec![Statement::Empty])))
        );
    }

    #[test]
    fn var_decl_inside_block() {
        assert_eq!(
            block_statement(&b"{\nmisal x = 5;\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(vec![Statement::VariableDeclaration(
                    VariableDeclaration {
                        id: Identifier {
                            name: String::from("x")
                        },
                        value: Expression::Literal(Literal::Number(5))
                    }
                )])
            ))
        );
    }

    #[test]
    fn recursive_empty_block() {
        assert_eq!(
            block_statement(&b"{\n{\n}\n}"[..]),
            Ok((
                &b""[..],
                Statement::BlockStatement(vec![Statement::BlockStatement(vec![Statement::Empty])])
            ))
        );
    }
}
