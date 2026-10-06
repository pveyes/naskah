use super::ast::*;
use super::lexer::{lex, ParseError, Tok, Token};

type Result<T> = std::result::Result<T, ParseError>;

const RESERVED: [&str; 16] = [
    "misal", "konstan", "jika", "lain", "selama", "ulang", "berhenti", "lanjut", "fungsi",
    "kembali", "benar", "salah", "kosong", "dan", "atau", "bukan",
];

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

fn describe(tok: &Tok) -> String {
    match tok {
        Tok::Ident(s) => format!("`{}`", s),
        Tok::Number(n) => format!("`{}`", n),
        Tok::Str(s) => format!("\"{}\"", s),
        Tok::Punct(p) => format!("`{}`", p),
        Tok::Eof => String::from("akhir kode"),
    }
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[i].tok
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn error<T>(&self, message: String) -> Result<T> {
        let t = &self.tokens[self.pos];
        Err(ParseError { message, line: t.line, col: t.col })
    }

    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Tok::Punct(q) if *q == p)
    }

    fn eat_punct(&mut self, p: &str) -> bool {
        if self.is_punct(p) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_punct(&mut self, p: &str) -> Result<()> {
        if self.eat_punct(p) {
            Ok(())
        } else {
            self.error(format!("diharapkan `{}`, ditemukan {}", p, describe(self.peek())))
        }
    }

    fn is_word(&self, w: &str) -> bool {
        match self.peek() {
            Tok::Ident(s) => s == w,
            _ => false,
        }
    }

    fn eat_word(&mut self, w: &str) -> bool {
        if self.is_word(w) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn identifier(&mut self) -> Result<Identifier> {
        match self.peek().clone() {
            Tok::Ident(name) if !RESERVED.contains(&name.as_str()) => {
                self.advance();
                Ok(Identifier { name })
            }
            Tok::Ident(name) => self.error(format!(
                "`{}` adalah kata kunci dan tidak bisa dipakai sebagai nama",
                name
            )),
            other => self.error(format!("diharapkan nama, ditemukan {}", describe(&other))),
        }
    }

    // statements

    fn program(&mut self) -> Result<Program> {
        let mut body = Vec::new();
        while *self.peek() != Tok::Eof {
            body.push(self.statement()?);
        }
        Ok(Program { body })
    }

    fn block(&mut self) -> Result<BlockStatement> {
        self.expect_punct("{")?;
        let mut statements = Vec::new();
        while !self.is_punct("}") {
            if *self.peek() == Tok::Eof {
                return self.error(String::from("diharapkan `}`, ditemukan akhir kode"));
            }
            statements.push(self.statement()?);
        }
        self.advance();
        Ok(BlockStatement {
            body: if statements.is_empty() { None } else { Some(statements) },
        })
    }

    fn statement(&mut self) -> Result<Statement> {
        if self.is_punct("{") {
            return Ok(Statement::BlockStatement(self.block()?));
        }

        if self.is_word("misal") || self.is_word("konstan") {
            let kind = if self.is_word("misal") { VariableKind::Let } else { VariableKind::Const };
            self.advance();
            let id = self.identifier()?;
            self.expect_punct("=")?;
            let value = self.expression()?;
            self.expect_punct(";")?;
            return Ok(Statement::VariableDeclaration(VariableDeclaration { kind, id, value }));
        }

        if self.eat_word("fungsi") {
            let id = self.identifier()?;
            self.expect_punct("(")?;
            let mut params = Vec::new();
            if !self.is_punct(")") {
                loop {
                    params.push(self.identifier()?);
                    if !self.eat_punct(",") {
                        break;
                    }
                }
            }
            self.expect_punct(")")?;
            let body = self.block()?;
            return Ok(Statement::FunctionDeclaration(FunctionDeclaration { id, params, body }));
        }

        if self.is_word("jika") {
            return Ok(Statement::IfStatement(self.if_statement()?));
        }

        if self.eat_word("selama") {
            let test = self.expression()?;
            let body = self.block()?;
            return Ok(Statement::While(WhileStatement { test, body }));
        }

        if self.eat_word("ulang") {
            return Ok(Statement::Loop(self.block()?));
        }

        if self.eat_word("berhenti") {
            self.expect_punct(";")?;
            return Ok(Statement::Break);
        }

        if self.eat_word("lanjut") {
            self.expect_punct(";")?;
            return Ok(Statement::Continue);
        }

        if self.eat_word("kembali") {
            let value = if self.is_punct(";") { None } else { Some(self.expression()?) };
            self.expect_punct(";")?;
            return Ok(Statement::Return(value));
        }

        let e = self.expression()?;
        self.expect_punct(";")?;
        Ok(Statement::Expression(e))
    }

    fn if_statement(&mut self) -> Result<IfStatement> {
        self.advance(); // jika
        let mut test = self.expression()?;

        // shorthand: `jika x benar {` means `jika x == benar {`
        for (word, literal) in &[("benar", true), ("salah", false)] {
            if self.eat_word(word) {
                test = equal(test, Literal::Boolean(*literal));
            }
        }
        if self.eat_word("kosong") {
            test = equal(test, Literal::Null);
        }

        let consequent = self.block()?;
        let alternate = if self.eat_word("lain") {
            if self.is_word("jika") {
                Some(AlternateStatement::IfStatement(Box::new(self.if_statement()?)))
            } else {
                Some(AlternateStatement::BlockStatement(self.block()?))
            }
        } else {
            None
        };

        Ok(IfStatement { test, consequent, alternate })
    }

    // expressions, from lowest to highest precedence:
    //   =  <  atau  <  dan  <  bukan  <  == !=  <  < > <= >=  <  + -  <  * / %  <  - (unary)  <  ^

    fn expression(&mut self) -> Result<Expression> {
        if let (Tok::Ident(name), Tok::Punct("=")) = (self.peek().clone(), self.peek_at(1).clone()) {
            if !RESERVED.contains(&name.as_str()) {
                self.advance();
                self.advance();
                let value = self.expression()?;
                return Ok(Expression::Assignment(AssignmentExpression {
                    id: Identifier { name },
                    value: Box::new(value),
                }));
            }
        }
        self.or()
    }

    fn or(&mut self) -> Result<Expression> {
        let mut left = self.and()?;
        while self.eat_word("atau") {
            let right = self.and()?;
            left = binary(left, Operator::Or, right);
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expression> {
        let mut left = self.not()?;
        while self.eat_word("dan") {
            let right = self.not()?;
            left = binary(left, Operator::And, right);
        }
        Ok(left)
    }

    fn not(&mut self) -> Result<Expression> {
        if self.eat_word("bukan") {
            let argument = self.not()?;
            return Ok(unary(UnaryOperator::Not, argument));
        }
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression> {
        let mut left = self.comparison()?;
        loop {
            let op = if self.eat_punct("==") {
                Operator::Equal
            } else if self.eat_punct("!=") {
                Operator::NotEqual
            } else {
                return Ok(left);
            };
            let right = self.comparison()?;
            left = binary(left, op, right);
        }
    }

    fn comparison(&mut self) -> Result<Expression> {
        let mut left = self.additive()?;
        loop {
            let op = if self.eat_punct(">=") {
                Operator::GreaterThanOrEqualTo
            } else if self.eat_punct("<=") {
                Operator::LessThanOrEqualTo
            } else if self.eat_punct(">") {
                Operator::GreaterThan
            } else if self.eat_punct("<") {
                Operator::LessThan
            } else {
                return Ok(left);
            };
            let right = self.additive()?;
            left = binary(left, op, right);
        }
    }

    fn additive(&mut self) -> Result<Expression> {
        let mut left = self.multiplicative()?;
        loop {
            let op = if self.eat_punct("+") {
                Operator::Addition
            } else if self.eat_punct("-") {
                Operator::Substraction
            } else {
                return Ok(left);
            };
            let right = self.multiplicative()?;
            left = binary(left, op, right);
        }
    }

    fn multiplicative(&mut self) -> Result<Expression> {
        let mut left = self.unary()?;
        loop {
            let op = if self.eat_punct("*") {
                Operator::Multiplication
            } else if self.eat_punct("/") {
                Operator::Division
            } else if self.eat_punct("%") {
                Operator::Remainder
            } else {
                return Ok(left);
            };
            let right = self.unary()?;
            left = binary(left, op, right);
        }
    }

    fn unary(&mut self) -> Result<Expression> {
        if self.eat_punct("-") {
            let argument = self.unary()?;
            return Ok(unary(UnaryOperator::Negate, argument));
        }
        self.power()
    }

    fn power(&mut self) -> Result<Expression> {
        let base = self.primary()?;
        if self.eat_punct("^") {
            // right associative, and the exponent may be negated: 2 ^ -1
            let exponent = self.unary()?;
            return Ok(binary(base, Operator::Exponentiation, exponent));
        }
        Ok(base)
    }

    fn primary(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Tok::Number(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Number(n)))
            }
            Tok::Str(s) => {
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            Tok::Punct("(") => {
                self.advance();
                let e = self.expression()?;
                self.expect_punct(")")?;
                Ok(e)
            }
            Tok::Ident(word) if word == "benar" || word == "salah" => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(word == "benar")))
            }
            Tok::Ident(word) if word == "kosong" => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }
            Tok::Ident(_) => {
                let callee = self.identifier()?;
                if !self.eat_punct("(") {
                    return Ok(Expression::Identifier(callee));
                }
                let mut arguments = Vec::new();
                if !self.is_punct(")") {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.eat_punct(",") {
                            break;
                        }
                    }
                }
                self.expect_punct(")")?;
                Ok(Expression::CallExpression(CallExpression { callee, arguments }))
            }
            other => self.error(format!("ekspresi tidak lengkap, ditemukan {}", describe(&other))),
        }
    }
}

fn binary(left: Expression, operator: Operator, right: Expression) -> Expression {
    Expression::BinaryExpression(Box::new(BinaryExpression { left, right, operator }))
}

fn unary(operator: UnaryOperator, argument: Expression) -> Expression {
    Expression::UnaryExpression(Box::new(UnaryExpression { operator, argument }))
}

fn equal(left: Expression, right: Literal) -> Expression {
    binary(left, Operator::Equal, Expression::Literal(right))
}

pub fn parse_program(src: &str) -> Result<Program> {
    let mut parser = Parser { tokens: lex(src)?, pos: 0 };
    parser.program()
}

#[cfg(test)]
mod test {
    use super::*;

    fn ok(src: &str) -> Vec<Statement> {
        parse_program(src).unwrap().body
    }

    fn err(src: &str) -> String {
        parse_program(src).unwrap_err().to_string()
    }

    fn num(n: f64) -> Expression {
        Expression::Literal(Literal::Number(n))
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier(Identifier { name: String::from(name) })
    }

    fn id(name: &str) -> Identifier {
        Identifier { name: String::from(name) }
    }

    fn bin(left: Expression, operator: Operator, right: Expression) -> Expression {
        binary(left, operator, right)
    }

    fn expr(src: &str) -> Expression {
        match ok(&format!("{};", src)).pop().unwrap() {
            Statement::Expression(e) => e,
            other => panic!("not an expression statement: {:?}", other),
        }
    }

    #[test]
    fn variable_declarations() {
        assert_eq!(
            ok("misal x = benar;"),
            vec![Statement::VariableDeclaration(VariableDeclaration {
                kind: VariableKind::Let,
                id: id("x"),
                value: Expression::Literal(Literal::Boolean(true)),
            })]
        );
        assert_eq!(
            ok("konstan pi=3.14;"),
            vec![Statement::VariableDeclaration(VariableDeclaration {
                kind: VariableKind::Const,
                id: id("pi"),
                value: num(3.14),
            })]
        );
    }

    #[test]
    fn literals() {
        assert_eq!(expr("kosong"), Expression::Literal(Literal::Null));
        assert_eq!(expr("0x1F"), num(31.0));
        assert_eq!(expr("0b101"), num(5.0));
        assert_eq!(expr("0d12"), num(12.0));
        assert_eq!(expr("0"), num(0.0));
        assert_eq!(
            expr("\"a \\\" b\""),
            Expression::Literal(Literal::String(String::from("a \\\" b")))
        );
    }

    #[test]
    fn arithmetic_precedence() {
        // 1 > 2 + 3  ==  1 > (2 + 3)
        assert_eq!(
            expr("1 > 2 + 3"),
            bin(num(1.0), Operator::GreaterThan, bin(num(2.0), Operator::Addition, num(3.0)))
        );
        // 1 + 2 * 3  ==  1 + (2 * 3)
        assert_eq!(
            expr("1 + 2 * 3"),
            bin(num(1.0), Operator::Addition, bin(num(2.0), Operator::Multiplication, num(3.0)))
        );
        // left associative: 1 - 2 - 3  ==  (1 - 2) - 3
        assert_eq!(
            expr("1 - 2 - 3"),
            bin(bin(num(1.0), Operator::Substraction, num(2.0)), Operator::Substraction, num(3.0))
        );
        // parentheses
        assert_eq!(
            expr("(1 + 2) * 3"),
            bin(bin(num(1.0), Operator::Addition, num(2.0)), Operator::Multiplication, num(3.0))
        );
    }

    #[test]
    fn exponent_is_right_associative_and_tighter_than_unary_minus() {
        assert_eq!(
            expr("2 ^ 3 ^ 2"),
            bin(num(2.0), Operator::Exponentiation, bin(num(3.0), Operator::Exponentiation, num(2.0)))
        );
        assert_eq!(
            expr("-2 ^ 2"),
            unary(UnaryOperator::Negate, bin(num(2.0), Operator::Exponentiation, num(2.0)))
        );
        assert_eq!(
            expr("2 ^ -1"),
            bin(num(2.0), Operator::Exponentiation, unary(UnaryOperator::Negate, num(1.0)))
        );
    }

    #[test]
    fn logic_operators() {
        // a atau b dan c  ==  a atau (b dan c)
        assert_eq!(
            expr("a atau b dan c"),
            bin(ident("a"), Operator::Or, bin(ident("b"), Operator::And, ident("c")))
        );
        // bukan binds looser than comparison: bukan a == b  ==  bukan (a == b)
        assert_eq!(
            expr("bukan a == b"),
            unary(UnaryOperator::Not, bin(ident("a"), Operator::Equal, ident("b")))
        );
        // ... but tighter than dan
        assert_eq!(
            expr("bukan a dan b"),
            bin(unary(UnaryOperator::Not, ident("a")), Operator::And, ident("b"))
        );
    }

    #[test]
    fn calls_and_assignment() {
        assert_eq!(
            expr("hello()"),
            Expression::CallExpression(CallExpression { callee: id("hello"), arguments: vec![] })
        );
        assert_eq!(
            expr("jumlah(1, x + 2)"),
            Expression::CallExpression(CallExpression {
                callee: id("jumlah"),
                arguments: vec![num(1.0), bin(ident("x"), Operator::Addition, num(2.0))],
            })
        );
        assert_eq!(
            expr("x = x ^ 5"),
            Expression::Assignment(AssignmentExpression {
                id: id("x"),
                value: Box::new(bin(ident("x"), Operator::Exponentiation, num(5.0))),
            })
        );
        // == is a comparison, not an assignment
        assert_eq!(expr("x == 1"), bin(ident("x"), Operator::Equal, num(1.0)));
    }

    #[test]
    fn blocks() {
        assert_eq!(
            ok("{\n}"),
            vec![Statement::BlockStatement(BlockStatement { body: None })]
        );
        assert_eq!(
            ok("{ { } }"),
            vec![Statement::BlockStatement(BlockStatement {
                body: Some(vec![Statement::BlockStatement(BlockStatement { body: None })])
            })]
        );
    }

    #[test]
    fn if_else_chain() {
        assert_eq!(
            ok("jika c == 2 {\n} lain jika d benar {\n} lain {\n}"),
            vec![Statement::IfStatement(IfStatement {
                test: bin(ident("c"), Operator::Equal, num(2.0)),
                consequent: BlockStatement { body: None },
                alternate: Some(AlternateStatement::IfStatement(Box::new(IfStatement {
                    test: bin(ident("d"), Operator::Equal, Expression::Literal(Literal::Boolean(true))),
                    consequent: BlockStatement { body: None },
                    alternate: Some(AlternateStatement::BlockStatement(BlockStatement { body: None })),
                }))),
            })]
        );
    }

    #[test]
    fn if_shorthand_equals_explicit_comparison() {
        assert_eq!(ok("jika a benar {\n}"), ok("jika a == benar {\n}"));
        assert_eq!(ok("jika a salah {\n} lain {\n}"), ok("jika a == salah {\n} lain {\n}"));
        assert_eq!(ok("jika a kosong {\n}"), ok("jika a == kosong {\n}"));
    }

    #[test]
    fn loops() {
        assert_eq!(
            ok("ulang {\nberhenti;\nlanjut;\n}"),
            vec![Statement::Loop(BlockStatement {
                body: Some(vec![Statement::Break, Statement::Continue])
            })]
        );
        assert_eq!(
            ok("selama x < 10 {\n x = x + 1;\n}"),
            vec![Statement::While(WhileStatement {
                test: bin(ident("x"), Operator::LessThan, num(10.0)),
                body: BlockStatement {
                    body: Some(vec![Statement::Expression(Expression::Assignment(
                        AssignmentExpression {
                            id: id("x"),
                            value: Box::new(bin(ident("x"), Operator::Addition, num(1.0))),
                        }
                    ))])
                },
            })]
        );
    }

    #[test]
    fn functions() {
        assert_eq!(
            ok("fungsi jumlah(a, b) {\n kembali a + b;\n}"),
            vec![Statement::FunctionDeclaration(FunctionDeclaration {
                id: id("jumlah"),
                params: vec![id("a"), id("b")],
                body: BlockStatement {
                    body: Some(vec![Statement::Return(Some(bin(
                        ident("a"),
                        Operator::Addition,
                        ident("b")
                    )))])
                },
            })]
        );
        assert_eq!(
            ok("fungsi diam() { kembali; }"),
            vec![Statement::FunctionDeclaration(FunctionDeclaration {
                id: id("diam"),
                params: vec![],
                body: BlockStatement { body: Some(vec![Statement::Return(None)]) },
            })]
        );
    }

    #[test]
    fn comments_are_ignored() {
        assert_eq!(
            ok("// halo\nmisal x = 1; // satu\n"),
            ok("misal x = 1;")
        );
        // division still works
        assert_eq!(expr("4 / 2"), bin(num(4.0), Operator::Division, num(2.0)));
    }

    #[test]
    fn errors_have_positions() {
        assert_eq!(err("misal x = 1"), "baris 1, kolom 12: diharapkan `;`, ditemukan akhir kode");
        assert_eq!(err("misal x = 1;\nmisal y = ;"), "baris 2, kolom 11: ekspresi tidak lengkap, ditemukan `;`");
        assert_eq!(err("misal jika = 1;"), "baris 1, kolom 7: `jika` adalah kata kunci dan tidak bisa dipakai sebagai nama");
        assert_eq!(err("jika x {"), "baris 1, kolom 9: diharapkan `}`, ditemukan akhir kode");
        assert_eq!(err("misal s = \"abc;"), "baris 1, kolom 11: teks tidak ditutup dengan `\"`");
        assert_eq!(err("misal x = 2x;"), "baris 1, kolom 11: angka tidak valid");
        assert_eq!(err("misal x = 1 @ 2;"), "baris 1, kolom 13: karakter `@` tidak dikenal");
    }
}
