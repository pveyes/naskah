use super::ast::*;
use super::lexer::{lex, scan_braces, ParseError, Tok, Token};

type Result<T> = std::result::Result<T, ParseError>;

// `setiap`, `dari`, `sampai`, `langkah`, `dalam` and `turunan` are only special
// in one place, so they stay usable as names.
const RESERVED: [&str; 24] = [
    "misal", "konstan", "jika", "lain", "selama", "ulang", "untuk", "pilih", "kalau", "berhenti",
    "lanjut", "fungsi", "hasilkan", "benar", "salah", "kosong", "dan", "atau", "bukan", "coba",
    "tangkap", "akhirnya", "lempar", "tunggu",
];

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// Set by `tunggu`; a function whose body sets it is async.
    saw_await: bool,
    /// Inside a class body, where `.nama` means the object itself.
    in_class: bool,
    /// How many `(`, `[` or `{` are open. Line breaks only end a statement outside of them.
    nesting: usize,
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

    fn tok_at(&self, i: usize) -> Option<&Tok> {
        self.tokens.get(i).map(|t| &t.tok)
    }

    /// Is the current token on the same line as the one before it? A token that
    /// starts a line cannot continue an expression, except inside brackets.
    fn same_line(&self) -> bool {
        self.nesting > 0 || self.pos == 0 || self.tokens[self.pos].line == self.tokens[self.pos - 1].line
    }

    /// An operator that has to sit on the same line as its left operand.
    fn eat_op(&mut self, p: &str) -> bool {
        self.same_line() && self.eat_punct(p)
    }

    /// A statement ends at `;`, at a line break, before `}`, or at the end of the code.
    fn end_statement(&mut self) -> Result<()> {
        if self.eat_punct(";") || self.is_punct("}") || *self.peek() == Tok::Eof || !self.same_line() {
            return Ok(());
        }
        self.error(format!(
            "diharapkan `;` atau baris baru, ditemukan {}",
            describe(self.peek())
        ))
    }

    /// Does `name(...)` here, optionally followed by `turunan Induk`, open a `{` on the same line?
    /// That is how a class and its methods are written.
    fn header_ahead(&self, allow_parent: bool) -> bool {
        if !matches!(self.tok_at(self.pos + 1), Some(Tok::Punct("("))) {
            return false;
        }
        let mut i = match self.after_parens(self.pos + 1) {
            Some(after) => after,
            None => return false,
        };
        if allow_parent && matches!(self.tok_at(i), Some(Tok::Ident(w)) if w == "turunan") {
            i += 1;
            if !matches!(self.tok_at(i), Some(Tok::Ident(_))) {
                return false;
            }
            i += 1;
            // the parent's arguments; without them the class is still a class, with a mistake in it
            if matches!(self.tok_at(i), Some(Tok::Punct("("))) {
                match self.after_parens(i) {
                    Some(after) => i = after,
                    None => return false,
                }
            }
        }
        matches!(self.tok_at(i), Some(Tok::Punct("{"))) && self.tokens[i].line == self.tokens[i - 1].line
    }

    /// Index of the token after the `)` that closes the `(` at `open`.
    fn after_parens(&self, open: usize) -> Option<usize> {
        let mut depth = 0;
        let mut i = open;
        loop {
            match self.tok_at(i)? {
                Tok::Punct("(") => depth += 1,
                Tok::Punct(")") => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                Tok::Eof => return None,
                _ => {}
            }
            i += 1;
        }
    }

    /// Does the `{` here start an object (`{}` or `{ nama: ...`) rather than a block?
    fn object_ahead(&self) -> bool {
        match (self.tok_at(self.pos + 1), self.tok_at(self.pos + 2)) {
            (Some(Tok::Punct("}")), _) => true,
            (Some(Tok::Ident(_)) | Some(Tok::Str(_)), Some(Tok::Punct(":"))) => true,
            _ => false,
        }
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

    /// A name this program declares (variable, function, parameter). Capital
    /// letters are reserved for classes.
    fn value_name(&mut self) -> Result<Identifier> {
        if let Tok::Ident(name) = self.peek() {
            if is_class_name(name) {
                return self.error(format!(
                    "`{}` diawali huruf kapital, padahal huruf kapital hanya untuk nama kelas",
                    name
                ));
            }
        }
        self.identifier()
    }

    fn class_name(&mut self) -> Result<Identifier> {
        if let Tok::Ident(name) = self.peek() {
            if !is_class_name(name) && !RESERVED.contains(&name.as_str()) {
                return self.error(format!(
                    "nama kelas harus diawali huruf kapital, ditemukan `{}`",
                    name
                ));
            }
        }
        self.identifier()
    }

    /// A function body, and whether it used `tunggu` and so has to be async.
    fn function_body(&mut self) -> Result<(BlockStatement, bool)> {
        let outer = std::mem::replace(&mut self.saw_await, false);
        let body = self.block();
        let is_async = std::mem::replace(&mut self.saw_await, outer);
        Ok((body?, is_async))
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
        // a block is its own world: line breaks end statements again, even inside brackets
        let outer_nesting = std::mem::replace(&mut self.nesting, 0);
        let mut statements = Vec::new();
        while !self.is_punct("}") {
            if *self.peek() == Tok::Eof {
                return self.error(String::from("diharapkan `}`, ditemukan akhir kode"));
            }
            statements.push(self.statement()?);
        }
        self.advance();
        self.nesting = outer_nesting;
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
            let id = self.value_name()?;
            self.expect_punct("=")?;
            let value = self.expression()?;
            self.end_statement()?;
            return Ok(Statement::VariableDeclaration(VariableDeclaration { kind, id, value }));
        }

        // `fungsi nama(` declares a function; `fungsi (` is a function value,
        // which falls through to the expression statement below
        if self.is_word("fungsi") && matches!(self.peek_at(1), Tok::Ident(_)) {
            self.advance();
            let id = self.value_name()?;
            let params = self.params()?;
            // a function declaration has a `this` of its own, so `.nama` is off limits inside
            let outer_class = std::mem::replace(&mut self.in_class, false);
            let body = self.function_body();
            self.in_class = outer_class;
            let (body, is_async) = body?;
            return Ok(Statement::FunctionDeclaration(FunctionDeclaration {
                id,
                params,
                body,
                is_async,
            }));
        }

        // `Hewan(nama) {` declares a class
        if matches!(self.peek(), Tok::Ident(w) if is_class_name(w)) && self.header_ahead(true) {
            return self.class_declaration();
        }
        // `hewan(nama) {` can only have been meant as one, so say what is wrong with the name
        if matches!(self.peek(), Tok::Ident(w) if !is_class_name(w) && !RESERVED.contains(&w.as_str()))
            && self.header_ahead(true)
        {
            self.class_name()?;
        }

        if self.eat_word("coba") {
            return self.try_statement();
        }

        if self.eat_word("lempar") {
            let value = self.expression()?;
            self.end_statement()?;
            return Ok(Statement::Throw(value));
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

        if self.eat_word("untuk") {
            return self.for_statement();
        }

        if self.eat_word("pilih") {
            return self.switch_statement();
        }

        if self.eat_word("berhenti") {
            self.end_statement()?;
            return Ok(Statement::Break);
        }

        if self.eat_word("lanjut") {
            self.end_statement()?;
            return Ok(Statement::Continue);
        }

        if self.eat_word("hasilkan") {
            let empty = self.is_punct(";")
                || self.is_punct("}")
                || *self.peek() == Tok::Eof
                || !self.same_line();
            let value = if empty { None } else { Some(self.expression()?) };
            self.end_statement()?;
            return Ok(Statement::Return(value));
        }

        let e = self.expression()?;
        self.end_statement()?;
        Ok(Statement::Expression(e))
    }

    /// `(a, b)`
    fn params(&mut self) -> Result<Vec<Identifier>> {
        self.expect_punct("(")?;
        self.comma_separated(")", Parser::value_name)
    }

    fn class_declaration(&mut self) -> Result<Statement> {
        let id = self.class_name()?;
        let params = self.params()?;
        let parent = if self.eat_word("turunan") { Some(self.parent_class()?) } else { None };
        self.expect_punct("{")?;

        let outer_class = std::mem::replace(&mut self.in_class, true);
        let outer_nesting = std::mem::replace(&mut self.nesting, 0);
        let outer_await = std::mem::replace(&mut self.saw_await, false);

        let mut body = Vec::new();
        let mut methods = Vec::new();
        while !self.is_punct("}") {
            if *self.peek() == Tok::Eof {
                return self.error(String::from("diharapkan `}`, ditemukan akhir kode"));
            }

            let is_method = matches!(self.peek(), Tok::Ident(w) if !is_class_name(w) && !RESERVED.contains(&w.as_str()))
                && self.header_ahead(false);
            if is_method {
                methods.push(self.method()?);
            } else {
                let (line, col) = (self.tokens[self.pos].line, self.tokens[self.pos].col);
                body.push(self.statement()?);
                // the statements of a class make up its constructor, which cannot be async
                if self.saw_await {
                    return Err(ParseError {
                        message: String::from(
                            "`tunggu` tidak bisa dipakai langsung di dalam kelas, pakai di dalam metode",
                        ),
                        line,
                        col,
                    });
                }
            }
        }
        self.advance(); // }

        self.in_class = outer_class;
        self.nesting = outer_nesting;
        self.saw_await = outer_await;
        Ok(Statement::ClassDeclaration(ClassDeclaration { id, params, parent, body, methods }))
    }

    /// `Hewan(nama)` after `turunan`: the arguments the parent's constructor is called with.
    fn parent_class(&mut self) -> Result<ParentClass> {
        let id = self.class_name()?;
        self.expect_punct("(")?;
        let (line, col) = (self.tokens[self.pos].line, self.tokens[self.pos].col);

        // they run before the object exists, so `.nama` is not available and neither is `tunggu`
        let outer_class = std::mem::replace(&mut self.in_class, false);
        let outer_await = std::mem::replace(&mut self.saw_await, false);
        let arguments = self.comma_separated(")", Parser::expression);
        let awaited = std::mem::replace(&mut self.saw_await, outer_await);
        self.in_class = outer_class;
        let arguments = arguments?;

        if awaited {
            return Err(ParseError {
                message: String::from(
                    "`tunggu` tidak bisa dipakai langsung di dalam kelas, pakai di dalam metode",
                ),
                line,
                col,
            });
        }
        Ok(ParentClass { id, arguments })
    }

    /// `suara() { ... }` inside a class
    fn method(&mut self) -> Result<Method> {
        let name = match self.peek().clone() {
            Tok::Ident(name) => name,
            other => {
                return self.error(format!("diharapkan nama metode, ditemukan {}", describe(&other)))
            }
        };
        self.advance();
        let params = self.params()?;
        let (body, is_async) = self.function_body()?;
        Ok(Method { name, params, body, is_async })
    }

    fn try_statement(&mut self) -> Result<Statement> {
        let block = self.block()?;

        let handler = if self.eat_word("tangkap") {
            let param = match self.peek() {
                Tok::Ident(_) => Some(self.value_name()?),
                _ => None,
            };
            Some(CatchClause { param, body: self.block()? })
        } else {
            None
        };
        let finalizer = if self.eat_word("akhirnya") { Some(self.block()?) } else { None };

        if handler.is_none() && finalizer.is_none() {
            return self.error(format!(
                "`coba` membutuhkan `tangkap` atau `akhirnya`, ditemukan {}",
                describe(self.peek())
            ));
        }
        Ok(Statement::Try(TryStatement { block, handler, finalizer }))
    }

    fn expect_word(&mut self, w: &str) -> Result<()> {
        if self.eat_word(w) {
            Ok(())
        } else {
            self.error(format!("diharapkan `{}`, ditemukan {}", w, describe(self.peek())))
        }
    }

    fn for_statement(&mut self) -> Result<Statement> {
        if self.eat_word("setiap") {
            let var = self.value_name()?;
            self.expect_word("dalam")?;
            let iterable = self.expression()?;
            let body = self.block()?;
            return Ok(Statement::ForEach(ForEachStatement { var, iterable, body }));
        }

        let var = self.value_name()?;
        self.expect_word("dari")?;
        let from = self.expression()?;
        self.expect_word("sampai")?;
        let to = self.expression()?;
        let step = if self.eat_word("langkah") { Some(self.expression()?) } else { None };
        let body = self.block()?;
        Ok(Statement::ForRange(ForRangeStatement { var, from, to, step, body }))
    }

    fn switch_statement(&mut self) -> Result<Statement> {
        let discriminant = self.expression()?;
        self.expect_punct("{")?;

        let mut cases = Vec::new();
        let mut default = None;
        loop {
            if self.eat_word("kalau") {
                let mut tests = vec![self.expression()?];
                while self.eat_punct(",") {
                    tests.push(self.expression()?);
                }
                let body = self.block()?;
                cases.push(SwitchCase { tests, body });
            } else if self.eat_word("lain") {
                default = Some(self.block()?);
                break;
            } else {
                break;
            }
        }

        if cases.is_empty() {
            return self.error(format!(
                "`pilih` membutuhkan minimal satu `kalau`, ditemukan {}",
                describe(self.peek())
            ));
        }
        self.expect_punct("}")?;
        Ok(Statement::Switch(SwitchStatement { discriminant, cases, default }))
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
        let left = self.or()?;
        if !(self.same_line() && self.is_punct("=")) {
            return Ok(left);
        }
        match left {
            Expression::Identifier(_) | Expression::Member(_) | Expression::Index(_) => {}
            _ => {
                return self.error(String::from(
                    "sebelah kiri `=` harus berupa variabel, properti, atau elemen daftar",
                ))
            }
        }
        self.advance();
        let value = self.expression()?;
        Ok(Expression::Assignment(AssignmentExpression {
            target: Box::new(left),
            value: Box::new(value),
        }))
    }

    fn or(&mut self) -> Result<Expression> {
        let mut left = self.and()?;
        while self.same_line() && self.eat_word("atau") {
            let right = self.and()?;
            left = binary(left, Operator::Or, right);
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expression> {
        let mut left = self.not()?;
        while self.same_line() && self.eat_word("dan") {
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
            let op = if self.eat_op("==") {
                Operator::Equal
            } else if self.eat_op("!=") {
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
            let op = if self.eat_op(">=") {
                Operator::GreaterThanOrEqualTo
            } else if self.eat_op("<=") {
                Operator::LessThanOrEqualTo
            } else if self.eat_op(">") {
                Operator::GreaterThan
            } else if self.eat_op("<") {
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
            let op = if self.eat_op("+") {
                Operator::Addition
            } else if self.eat_op("-") {
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
            let op = if self.eat_op("*") {
                Operator::Multiplication
            } else if self.eat_op("/") {
                Operator::Division
            } else if self.eat_op("%") {
                Operator::Remainder
            } else {
                return Ok(left);
            };
            let right = self.unary()?;
            left = binary(left, op, right);
        }
    }

    fn unary(&mut self) -> Result<Expression> {
        if self.eat_word("tunggu") {
            self.saw_await = true;
            let argument = self.unary()?;
            return Ok(Expression::Await(Box::new(argument)));
        }
        if self.eat_punct("-") {
            let argument = self.unary()?;
            return Ok(unary(UnaryOperator::Negate, argument));
        }
        self.power()
    }

    fn power(&mut self) -> Result<Expression> {
        let base = self.postfix()?;
        if self.eat_op("^") {
            // right associative, and the exponent may be negated: 2 ^ -1
            let exponent = self.unary()?;
            return Ok(binary(base, Operator::Exponentiation, exponent));
        }
        Ok(base)
    }

    /// `.name`, `[index]` and `(arguments)` after an expression.
    fn postfix(&mut self) -> Result<Expression> {
        let mut e = self.primary()?;
        loop {
            if self.eat_op(".") {
                match self.peek().clone() {
                    Tok::Ident(property) => {
                        self.advance();
                        e = Expression::Member(Box::new(MemberExpression { object: e, property }));
                    }
                    other => {
                        return self.error(format!(
                            "diharapkan nama properti, ditemukan {}",
                            describe(&other)
                        ))
                    }
                }
            } else if self.eat_op("[") {
                let index = self.expression()?;
                self.expect_punct("]")?;
                e = Expression::Index(Box::new(IndexExpression { object: e, index }));
            } else if self.same_line() && self.is_punct("(") {
                match e {
                    Expression::Identifier(_)
                    | Expression::Member(_)
                    | Expression::Index(_)
                    | Expression::CallExpression(_) => {}
                    _ => return self.error(String::from("ini bukan fungsi dan tidak bisa dipanggil")),
                }
                let line = self.tokens[self.pos].line;
                self.advance();
                let arguments = self.comma_separated(")", Parser::expression)?;
                // Capital letters mean a class, and calling a class always builds one
                e = if is_class_callee(&e) {
                    Expression::New(Box::new(NewExpression { callee: e, arguments }))
                } else {
                    Expression::CallExpression(CallExpression { callee: Box::new(e), arguments, line })
                };
            } else {
                return Ok(e);
            }
        }
    }

    /// Items separated by `,` up to and including `close`. A trailing comma is fine.
    fn comma_separated<T>(
        &mut self,
        close: &str,
        item: fn(&mut Parser) -> Result<T>,
    ) -> Result<Vec<T>> {
        self.nesting += 1;
        let mut items = Vec::new();
        while !self.is_punct(close) {
            items.push(item(self)?);
            if !self.eat_punct(",") {
                break;
            }
        }
        self.expect_punct(close)?;
        self.nesting -= 1;
        Ok(items)
    }

    fn property(&mut self) -> Result<Property> {
        let key = match self.peek().clone() {
            Tok::Ident(name) => PropertyKey::Name(name),
            Tok::Str(text) => PropertyKey::Text(text),
            other => {
                return self.error(format!(
                    "diharapkan nama properti, ditemukan {}",
                    describe(&other)
                ))
            }
        };
        self.advance();
        self.expect_punct(":")?;
        let value = self.expression()?;
        Ok(Property { key, value })
    }

    fn primary(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Tok::Number(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Number(n)))
            }
            Tok::Str(raw) => {
                let (line, col) = (self.tokens[self.pos].line, self.tokens[self.pos].col);
                self.advance();
                let (e, awaited) = string_or_template(&raw, line, col, self.in_class)?;
                self.saw_await |= awaited;
                Ok(e)
            }
            Tok::Punct("(") => {
                self.advance();
                self.nesting += 1;
                let e = self.expression()?;
                self.expect_punct(")")?;
                self.nesting -= 1;
                Ok(e)
            }
            // `.nama` is the object a class method is running on
            Tok::Punct(".") => {
                if !self.in_class {
                    return self.error(String::from(
                        "`.nama` hanya bisa dipakai di dalam kelas, sebagai objek yang sedang dijalankan",
                    ));
                }
                let dot = self.pos;
                self.advance();
                // `..nama` is the parent's version of `.nama`; the two dots touch
                let parent = self.is_punct(".")
                    && self.tokens[self.pos].line == self.tokens[dot].line
                    && self.tokens[self.pos].col == self.tokens[dot].col + 1;
                if parent {
                    self.advance();
                }
                match self.peek().clone() {
                    Tok::Ident(property) => {
                        self.advance();
                        let object = if parent { Expression::Super } else { Expression::This };
                        Ok(Expression::Member(Box::new(MemberExpression { object, property })))
                    }
                    other => self.error(format!(
                        "diharapkan nama properti, ditemukan {}",
                        describe(&other)
                    )),
                }
            }
            Tok::Punct("[") => {
                self.advance();
                Ok(Expression::List(self.comma_separated("]", Parser::expression)?))
            }
            Tok::Punct("{") if self.object_ahead() => {
                self.advance();
                Ok(Expression::Object(self.comma_separated("}", Parser::property)?))
            }
            // any other `{` is a block that produces a value with `hasilkan`
            Tok::Punct("{") => {
                let (body, is_async) = self.function_body()?;
                if is_async {
                    // awaiting the block is awaiting inside the function around it
                    self.saw_await = true;
                }
                Ok(Expression::Block(Box::new(BlockExpression { body, is_async })))
            }
            Tok::Ident(word) if word == "benar" || word == "salah" => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(word == "benar")))
            }
            Tok::Ident(word) if word == "kosong" => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }
            Tok::Ident(word) if word == "fungsi" => {
                self.advance();
                let params = self.params()?;
                let (body, is_async) = self.function_body()?;
                Ok(Expression::Function(Box::new(FunctionExpression { params, body, is_async })))
            }
            Tok::Ident(_) => Ok(Expression::Identifier(self.identifier()?)),
            other => self.error(format!("ekspresi tidak lengkap, ditemukan {}", describe(&other))),
        }
    }
}

/// Capital letters are for classes: `Hewan`, `Galat`, `Date`.
fn is_class_name(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c.is_uppercase())
}

/// `Hewan(...)` and `a.Hewan(...)` build an object.
fn is_class_callee(e: &Expression) -> bool {
    match e {
        Expression::Identifier(i) => is_class_name(&i.name),
        Expression::Member(m) => is_class_name(&m.property),
        _ => false,
    }
}

fn shift(mut e: ParseError, line: usize, col: usize) -> ParseError {
    if e.line == 1 {
        e.col += col - 1;
    }
    e.line += line - 1;
    e
}

/// Parse the expression between `{` and `}` of a text. `line`/`col` locate its
/// first character in the whole source so errors point at the right place.
fn embedded(src: &str, line: usize, col: usize, in_class: bool) -> Result<(Expression, bool)> {
    if src.trim().is_empty() {
        return Err(ParseError {
            message: String::from("ekspresi di dalam `{ }` kosong, tulis `\\{` untuk kurung biasa"),
            line,
            col,
        });
    }

    let mut tokens = lex(src).map_err(|e| shift(e, line, col))?;
    for t in &mut tokens {
        if t.line == 1 {
            t.col += col - 1;
        }
        t.line += line - 1;
    }

    let mut parser = Parser { tokens, pos: 0, saw_await: false, in_class, nesting: 1 };
    let e = parser.expression()?;
    if *parser.peek() != Tok::Eof {
        return parser.error(format!(
            "diharapkan akhir ekspresi, ditemukan {}",
            describe(parser.peek())
        ));
    }
    Ok((e, parser.saw_await))
}

/// A text with `{ekspresi}` in it becomes a template, anything else a plain string.
/// `line`/`col` is where the opening quote is.
fn string_or_template(raw: &str, line: usize, col: usize, in_class: bool) -> Result<(Expression, bool)> {
    let chars: Vec<char> = raw.chars().collect();
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut awaited = false;
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\\' => {
                text.push('\\');
                if let Some(next) = chars.get(i + 1) {
                    text.push(*next);
                }
                i += 2;
            }
            '{' => {
                let end = scan_braces(&chars, i).map_err(|_| ParseError {
                    message: String::from("tanda `{` di dalam teks tidak ditutup"),
                    line,
                    col,
                })?;
                let inner: String = chars[i + 1..end - 1].iter().collect();
                if !text.is_empty() {
                    parts.push(TemplatePart::Text(std::mem::take(&mut text)));
                }
                // +1 skips the opening quote, +1 skips the `{`
                let (e, inner_awaited) = embedded(&inner, line, col + 1 + i + 1, in_class)?;
                awaited |= inner_awaited;
                parts.push(TemplatePart::Expression(e));
                i = end;
            }
            c => {
                text.push(c);
                i += 1;
            }
        }
    }

    if parts.is_empty() {
        return Ok((Expression::Literal(Literal::String(String::from(raw))), false));
    }
    if !text.is_empty() {
        parts.push(TemplatePart::Text(text));
    }
    Ok((Expression::Template(parts), awaited))
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
    let mut parser = Parser { tokens: lex(src)?, pos: 0, saw_await: false, in_class: false, nesting: 0 };
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

    fn call(name: &str, arguments: Vec<Expression>) -> Expression {
        Expression::CallExpression(CallExpression {
            callee: Box::new(ident(name)),
            arguments,
            line: 1,
        })
    }

    fn assign(target: Expression, value: Expression) -> Expression {
        Expression::Assignment(AssignmentExpression {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    fn member(object: Expression, property: &str) -> Expression {
        Expression::Member(Box::new(MemberExpression { object, property: String::from(property) }))
    }

    fn index(object: Expression, index: Expression) -> Expression {
        Expression::Index(Box::new(IndexExpression { object, index }))
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
            call("hello", vec![])
        );
        assert_eq!(
            expr("jumlah(1, x + 2)"),
            call("jumlah", vec![num(1.0), bin(ident("x"), Operator::Addition, num(2.0))])
        );
        assert_eq!(
            expr("x = x ^ 5"),
            assign(ident("x"), bin(ident("x"), Operator::Exponentiation, num(5.0)))
        );
        // == is a comparison, not an assignment
        assert_eq!(expr("x == 1"), bin(ident("x"), Operator::Equal, num(1.0)));
    }

    #[test]
    fn calls_remember_their_line() {
        let body = ok("\n\ntulis(1);\nmisal x = f(\n  2\n);");
        match &body[0] {
            Statement::Expression(Expression::CallExpression(c)) => assert_eq!(c.line, 3),
            other => panic!("{:?}", other),
        }
        match &body[1] {
            Statement::VariableDeclaration(v) => match &v.value {
                Expression::CallExpression(c) => assert_eq!(c.line, 4),
                other => panic!("{:?}", other),
            },
            other => panic!("{:?}", other),
        }
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
                    body: Some(vec![Statement::Expression(assign(
                        ident("x"),
                        bin(ident("x"), Operator::Addition, num(1.0))
                    ))])
                },
            })]
        );
    }

    #[test]
    fn functions() {
        assert_eq!(
            ok("fungsi jumlah(a, b) {\n hasilkan a + b;\n}"),
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
                is_async: false,
            })]
        );
        assert_eq!(
            ok("fungsi diam() { hasilkan; }"),
            vec![Statement::FunctionDeclaration(FunctionDeclaration {
                id: id("diam"),
                params: vec![],
                body: BlockStatement { body: Some(vec![Statement::Return(None)]) },
                is_async: false,
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

    fn text(s: &str) -> Expression {
        Expression::Literal(Literal::String(String::from(s)))
    }

    fn block(statements: Vec<Statement>) -> BlockStatement {
        BlockStatement { body: if statements.is_empty() { None } else { Some(statements) } }
    }

    #[test]
    fn lists_and_objects() {
        assert_eq!(expr("[]"), Expression::List(vec![]));
        assert_eq!(expr("[1, x + 1, ]"), Expression::List(vec![
            num(1.0),
            bin(ident("x"), Operator::Addition, num(1.0)),
        ]));
        assert_eq!(expr("({})"), Expression::Object(vec![]));
        assert_eq!(
            expr("({ nama: \"Budi\", \"umur anak\": [1], })"),
            Expression::Object(vec![
                Property { key: PropertyKey::Name(String::from("nama")), value: text("Budi") },
                Property {
                    key: PropertyKey::Text(String::from("umur anak")),
                    value: Expression::List(vec![num(1.0)]),
                },
            ])
        );
    }

    #[test]
    fn member_index_and_call_chain() {
        assert_eq!(expr("a.b"), member(ident("a"), "b"));
        assert_eq!(expr("a[0]"), index(ident("a"), num(0.0)));
        assert_eq!(
            expr("a.b[1].c"),
            member(index(member(ident("a"), "b"), num(1.0)), "c")
        );
        assert_eq!(
            expr("daftar.tambah(4)"),
            Expression::CallExpression(CallExpression {
                callee: Box::new(member(ident("daftar"), "tambah")),
                arguments: vec![num(4.0)],
                line: 1,
            })
        );
        // calling the result of a call
        assert_eq!(
            expr("f(1)(2)"),
            Expression::CallExpression(CallExpression {
                callee: Box::new(call("f", vec![num(1.0)])),
                arguments: vec![num(2.0)],
                line: 1,
            })
        );
        // postfix binds tighter than unary minus and ^
        assert_eq!(
            expr("-a.b ^ 2"),
            unary(
                UnaryOperator::Negate,
                bin(member(ident("a"), "b"), Operator::Exponentiation, num(2.0))
            )
        );
    }

    #[test]
    fn assignment_targets() {
        assert_eq!(expr("a[0] = 1"), assign(index(ident("a"), num(0.0)), num(1.0)));
        assert_eq!(expr("o.x = o.y = 2"), assign(
            member(ident("o"), "x"),
            assign(member(ident("o"), "y"), num(2.0)),
        ));
        assert_eq!(
            err("1 + 2 = 3;"),
            "baris 1, kolom 7: sebelah kiri `=` harus berupa variabel, properti, atau elemen daftar"
        );
        assert_eq!(
            err("f() = 3;"),
            "baris 1, kolom 5: sebelah kiri `=` harus berupa variabel, properti, atau elemen daftar"
        );
    }

    #[test]
    fn calling_a_non_function_is_an_error() {
        assert_eq!(err("5(3);"), "baris 1, kolom 2: ini bukan fungsi dan tidak bisa dipanggil");
        assert_eq!(err("[1](2);"), "baris 1, kolom 4: ini bukan fungsi dan tidak bisa dipanggil");
    }

    #[test]
    fn block_versus_object() {
        // `{` starting a statement is a block, inside an expression it is an object
        assert_eq!(ok("{ }"), vec![Statement::BlockStatement(BlockStatement { body: None })]);
        assert_eq!(
            ok("jika x { }"),
            vec![Statement::IfStatement(IfStatement {
                test: ident("x"),
                consequent: BlockStatement { body: None },
                alternate: None,
            })]
        );
        assert_eq!(
            ok("misal o = { a: 1 };"),
            vec![Statement::VariableDeclaration(VariableDeclaration {
                kind: VariableKind::Let,
                id: id("o"),
                value: Expression::Object(vec![Property {
                    key: PropertyKey::Name(String::from("a")),
                    value: num(1.0),
                }]),
            })]
        );
    }

    #[test]
    fn for_range() {
        assert_eq!(
            ok("untuk i dari 1 sampai 10 { }"),
            vec![Statement::ForRange(ForRangeStatement {
                var: id("i"),
                from: num(1.0),
                to: num(10.0),
                step: None,
                body: block(vec![]),
            })]
        );
        assert_eq!(
            ok("untuk i dari n sampai 0 langkah -2 { berhenti; }"),
            vec![Statement::ForRange(ForRangeStatement {
                var: id("i"),
                from: ident("n"),
                to: num(0.0),
                step: Some(unary(UnaryOperator::Negate, num(2.0))),
                body: block(vec![Statement::Break]),
            })]
        );
        assert_eq!(
            err("untuk i dari 1 { }"),
            "baris 1, kolom 16: diharapkan `sampai`, ditemukan `{`"
        );
    }

    #[test]
    fn for_each() {
        assert_eq!(
            ok("untuk setiap x dalam [1, 2] { lanjut; }"),
            vec![Statement::ForEach(ForEachStatement {
                var: id("x"),
                iterable: Expression::List(vec![num(1.0), num(2.0)]),
                body: block(vec![Statement::Continue]),
            })]
        );
        assert_eq!(
            err("untuk setiap x daftar { }"),
            "baris 1, kolom 16: diharapkan `dalam`, ditemukan `daftar`"
        );
    }

    #[test]
    fn contextual_words_are_still_names() {
        assert_eq!(expr("dari + sampai"), bin(ident("dari"), Operator::Addition, ident("sampai")));
        assert!(parse_program("misal setiap = 1;").is_ok());
        assert!(parse_program("misal untuk = 1;").is_err());
    }

    #[test]
    fn switch() {
        assert_eq!(
            ok("pilih x {\n kalau 1, 2 { berhenti; }\n kalau \"a\" { }\n lain { lanjut; }\n}"),
            vec![Statement::Switch(SwitchStatement {
                discriminant: ident("x"),
                cases: vec![
                    SwitchCase { tests: vec![num(1.0), num(2.0)], body: block(vec![Statement::Break]) },
                    SwitchCase { tests: vec![text("a")], body: block(vec![]) },
                ],
                default: Some(block(vec![Statement::Continue])),
            })]
        );
        assert_eq!(
            err("pilih x { }"),
            "baris 1, kolom 11: `pilih` membutuhkan minimal satu `kalau`, ditemukan `}`"
        );
        // lain has to be last
        assert_eq!(
            err("pilih x { kalau 1 { } lain { } kalau 2 { } }"),
            "baris 1, kolom 32: diharapkan `}`, ditemukan `kalau`"
        );
    }

    fn tpl(parts: Vec<TemplatePart>) -> Expression {
        Expression::Template(parts)
    }

    fn t_text(s: &str) -> TemplatePart {
        TemplatePart::Text(String::from(s))
    }

    fn t_expr(e: Expression) -> TemplatePart {
        TemplatePart::Expression(e)
    }

    #[test]
    fn text_templates() {
        assert_eq!(
            expr("\"Halo, {nama}!\""),
            tpl(vec![t_text("Halo, "), t_expr(ident("nama")), t_text("!")])
        );
        assert_eq!(
            expr("\"{a}{b + 1}\""),
            tpl(vec![
                t_expr(ident("a")),
                t_expr(bin(ident("b"), Operator::Addition, num(1.0))),
            ])
        );
        // strings and braces inside an interpolation
        assert_eq!(
            expr("\"x {f(\"}\")} y\""),
            tpl(vec![t_text("x "), t_expr(call("f", vec![text("}")])), t_text(" y")])
        );
        assert_eq!(
            expr("\"{ {a: 1}.a }\""),
            tpl(vec![t_expr(member(
                Expression::Object(vec![Property {
                    key: PropertyKey::Name(String::from("a")),
                    value: num(1.0),
                }]),
                "a"
            ))])
        );
        // an escaped brace is plain text and the string stays a literal
        assert_eq!(expr("\"\\{a}\""), text("\\{a}"));
        assert_eq!(expr("\"a } b\""), text("a } b"));
    }

    #[test]
    fn template_errors_point_inside_the_text() {
        assert_eq!(
            err("misal s = \"hai {1 +}\";"),
            "baris 1, kolom 20: ekspresi tidak lengkap, ditemukan akhir kode"
        );
        assert_eq!(
            err("\n\n  tulis(\"a {x y}\");"),
            "baris 3, kolom 15: diharapkan akhir ekspresi, ditemukan `y`"
        );
        assert_eq!(
            err("misal s = \"a {}\";"),
            "baris 1, kolom 15: ekspresi di dalam `{ }` kosong, tulis `\\{` untuk kurung biasa"
        );
        assert_eq!(
            err("misal s = \"a {b\";"),
            "baris 1, kolom 11: tanda `{` di dalam teks tidak ditutup, tulis `\\{` untuk kurung biasa"
        );
    }

    #[test]
    fn function_values() {
        assert_eq!(
            expr("xs.peta(fungsi (x) { hasilkan x; })"),
            Expression::CallExpression(CallExpression {
                callee: Box::new(member(ident("xs"), "peta")),
                arguments: vec![Expression::Function(Box::new(FunctionExpression {
                    params: vec![id("x")],
                    body: block(vec![Statement::Return(Some(ident("x")))]),
                    is_async: false,
                }))],
                line: 1,
            })
        );
        assert_eq!(
            ok("misal f = fungsi () { };"),
            vec![Statement::VariableDeclaration(VariableDeclaration {
                kind: VariableKind::Let,
                id: id("f"),
                value: Expression::Function(Box::new(FunctionExpression {
                    params: vec![],
                    body: block(vec![]),
                    is_async: false,
                })),
            })]
        );
        // a name after `fungsi` is a declaration, so it is not allowed in an expression
        assert_eq!(
            err("misal f = fungsi g() { };"),
            "baris 1, kolom 18: diharapkan `(`, ditemukan `g`"
        );
    }

    #[test]
    fn functions_that_use_tunggu_are_async() {
        assert_eq!(
            ok("fungsi ambil() { tunggu f(); }"),
            vec![Statement::FunctionDeclaration(FunctionDeclaration {
                id: id("ambil"),
                params: vec![],
                body: block(vec![Statement::Expression(Expression::Await(Box::new(call(
                    "f",
                    vec![]
                ))))]),
                is_async: true,
            })]
        );

        // no tunggu, no async
        match &ok("fungsi f() { }")[0] {
            Statement::FunctionDeclaration(f) => assert!(!f.is_async),
            other => panic!("{:?}", other),
        }
        // tunggu in a nested function only makes that function async
        match &ok("fungsi luar() { misal d = fungsi () { tunggu f(); }; }")[0] {
            Statement::FunctionDeclaration(f) => assert!(!f.is_async),
            other => panic!("{:?}", other),
        }
        // ... and one inside a text counts for the function around it
        match &ok("fungsi f() { tulis(\"{tunggu g()}\"); }")[0] {
            Statement::FunctionDeclaration(f) => assert!(f.is_async),
            other => panic!("{:?}", other),
        }
        match expr("fungsi (a) { tunggu f(); }") {
            Expression::Function(f) => assert!(f.is_async),
            other => panic!("{:?}", other),
        }
        match expr("fungsi (a) { }") {
            Expression::Function(f) => assert!(!f.is_async),
            other => panic!("{:?}", other),
        }

        assert_eq!(expr("tunggu f()"), Expression::Await(Box::new(call("f", vec![]))));
        // tunggu binds like a unary operator: (tunggu a) + b
        assert_eq!(
            expr("tunggu a + b"),
            bin(Expression::Await(Box::new(ident("a"))), Operator::Addition, ident("b"))
        );
    }

    #[test]
    fn try_catch_finally() {
        assert_eq!(
            ok("coba { lanjut; } tangkap galat { berhenti; } akhirnya { }"),
            vec![Statement::Try(TryStatement {
                block: block(vec![Statement::Continue]),
                handler: Some(CatchClause {
                    param: Some(id("galat")),
                    body: block(vec![Statement::Break]),
                }),
                finalizer: Some(block(vec![])),
            })]
        );
        assert_eq!(
            ok("coba { } tangkap { }"),
            vec![Statement::Try(TryStatement {
                block: block(vec![]),
                handler: Some(CatchClause { param: None, body: block(vec![]) }),
                finalizer: None,
            })]
        );
        assert_eq!(
            err("coba { } x;"),
            "baris 1, kolom 10: `coba` membutuhkan `tangkap` atau `akhirnya`, ditemukan `x`"
        );
    }

    fn new_of(callee: Expression, arguments: Vec<Expression>) -> Expression {
        Expression::New(Box::new(NewExpression { callee, arguments }))
    }

    #[test]
    fn capitalised_calls_build_objects() {
        assert_eq!(
            ok("lempar Galat(\"x\");"),
            vec![Statement::Throw(new_of(ident("Galat"), vec![text("x")]))]
        );
        assert_eq!(expr("hewan(1)"), call("hewan", vec![num(1.0)]));
        assert_eq!(expr("Hewan(1)"), new_of(ident("Hewan"), vec![num(1.0)]));
        assert_eq!(
            expr("a.B(1).c"),
            member(new_of(member(ident("a"), "B"), vec![num(1.0)]), "c")
        );
        assert_eq!(expr("Hewan(1).suara()").clone(), {
            Expression::CallExpression(CallExpression {
                callee: Box::new(member(new_of(ident("Hewan"), vec![num(1.0)]), "suara")),
                arguments: vec![],
                line: 1,
            })
        });
        // a lowercase method on a capitalised object is a normal call
        assert_eq!(
            expr("Math.max(1)"),
            Expression::CallExpression(CallExpression {
                callee: Box::new(member(ident("Math"), "max")),
                arguments: vec![num(1.0)],
                line: 1,
            })
        );
        // naming a class without calling it is just a value
        assert_eq!(expr("Hewan"), ident("Hewan"));
    }

    #[test]
    fn capital_letters_are_only_for_classes() {
        let capital = |name: &str| {
            format!("`{}` diawali huruf kapital, padahal huruf kapital hanya untuk nama kelas", name)
        };
        assert_eq!(err("misal Nama = 1;"), format!("baris 1, kolom 7: {}", capital("Nama")));
        assert_eq!(err("fungsi Tambah() { }"), format!("baris 1, kolom 8: {}", capital("Tambah")));
        assert_eq!(err("fungsi f(Ab) { }"), format!("baris 1, kolom 10: {}", capital("Ab")));
        assert_eq!(err("untuk Ab dari 1 sampai 2 { }"), format!("baris 1, kolom 7: {}", capital("Ab")));
        assert_eq!(err("untuk setiap Ab dalam xs { }"), format!("baris 1, kolom 14: {}", capital("Ab")));
        assert_eq!(err("coba { } tangkap E { }"), format!("baris 1, kolom 18: {}", capital("E")));

        // a lowercase name followed by `(...) {` was meant to be a class
        assert_eq!(
            err("hewan(nama) { }"),
            "baris 1, kolom 1: nama kelas harus diawali huruf kapital, ditemukan `hewan`"
        );
        assert_eq!(
            err("Hewan(nama) turunan orang(nama) { }"),
            "baris 1, kolom 21: nama kelas harus diawali huruf kapital, ditemukan `orang`"
        );
        assert_eq!(err("Hewan(Ab) { }"), format!("baris 1, kolom 7: {}", capital("Ab")));

        // underscores are neither
        assert!(parse_program("misal _x = 1;").is_ok());
    }

    fn let_(name: &str, value: Expression) -> Statement {
        Statement::VariableDeclaration(VariableDeclaration {
            kind: VariableKind::Let,
            id: id(name),
            value,
        })
    }

    fn this_member(property: &str) -> Expression {
        member(Expression::This, property)
    }

    fn block_value(statements: Vec<Statement>, is_async: bool) -> Expression {
        Expression::Block(Box::new(BlockExpression { body: block(statements), is_async }))
    }

    #[test]
    fn classes() {
        let body = ok("Kucing(nama) turunan Hewan(nama, 1) {
  .suara = 1
  ambil() { tunggu g() }
  suara() { .ambil() }
}");
        assert_eq!(
            body,
            vec![Statement::ClassDeclaration(ClassDeclaration {
                id: id("Kucing"),
                params: vec![id("nama")],
                parent: Some(ParentClass {
                    id: id("Hewan"),
                    arguments: vec![ident("nama"), num(1.0)],
                }),
                body: vec![Statement::Expression(assign(this_member("suara"), num(1.0)))],
                methods: vec![
                    Method {
                        name: String::from("ambil"),
                        params: vec![],
                        body: block(vec![Statement::Expression(Expression::Await(Box::new(
                            Expression::CallExpression(CallExpression {
                                callee: Box::new(ident("g")),
                                arguments: vec![],
                                line: 3,
                            })
                        )))]),
                        is_async: true,
                    },
                    Method {
                        name: String::from("suara"),
                        params: vec![],
                        body: block(vec![Statement::Expression(Expression::CallExpression(
                            CallExpression {
                                callee: Box::new(this_member("ambil")),
                                arguments: vec![],
                                line: 4,
                            }
                        ))]),
                        is_async: false,
                    },
                ],
            })]
        );
        assert_eq!(
            ok("Hewan() { }"),
            vec![Statement::ClassDeclaration(ClassDeclaration {
                id: id("Hewan"),
                params: vec![],
                parent: None,
                body: vec![],
                methods: vec![],
            })]
        );
        // a parent with no arguments still needs its parentheses
        match &ok("A() turunan B() { }")[0] {
            Statement::ClassDeclaration(c) => {
                assert_eq!(c.parent, Some(ParentClass { id: id("B"), arguments: vec![] }))
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn a_class_header_is_a_name_with_params_and_a_brace_on_the_same_line() {
        // not a class: a call, and a block on the next line
        assert_eq!(ok("Hewan(1)\n{ }").len(), 2);
        assert!(matches!(&ok("Hewan(1)\n{ }")[0], Statement::Expression(Expression::New(_))));
        // a class nested in a class
        match &ok("A() { B() { } }")[0] {
            Statement::ClassDeclaration(c) => {
                assert!(matches!(&c.body[0], Statement::ClassDeclaration(_)))
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn a_block_can_make_the_value_of_a_field() {
        assert_eq!(
            ok("Hewan(nama) {\n  .nama = { misal x = 5; hasilkan nama + x }\n}"),
            vec![Statement::ClassDeclaration(ClassDeclaration {
                id: id("Hewan"),
                params: vec![id("nama")],
                parent: None,
                body: vec![Statement::Expression(assign(
                    this_member("nama"),
                    block_value(
                        vec![
                            let_("x", num(5.0)),
                            Statement::Return(Some(bin(
                                ident("nama"),
                                Operator::Addition,
                                ident("x")
                            ))),
                        ],
                        false
                    )
                ))],
                methods: vec![],
            })]
        );
    }

    #[test]
    fn dot_name_is_only_for_classes() {
        let outside =
            "`.nama` hanya bisa dipakai di dalam kelas, sebagai objek yang sedang dijalankan";
        assert_eq!(err(".x = 1"), format!("baris 1, kolom 1: {}", outside));
        // a function declaration has a `this` of its own
        assert_eq!(
            err("Hewan() { fungsi f() { .x = 1 } }"),
            format!("baris 1, kolom 24: {}", outside)
        );
        // a function value keeps the object, and so do texts
        assert!(parse_program("Hewan() { m() { misal f = fungsi () { hasilkan .x }\n tulis(\"{.x}\") } }").is_ok());
        // `.` needs a name
        assert_eq!(
            err("Hewan() { .1 }"),
            "baris 1, kolom 12: diharapkan nama properti, ditemukan `1`"
        );
    }

    #[test]
    fn two_dots_are_the_parent() {
        let body = ok("Kucing() turunan Hewan() {\n  suara() { ..suara() }\n}");
        match &body[0] {
            Statement::ClassDeclaration(c) => assert_eq!(
                c.methods[0].body,
                block(vec![Statement::Expression(Expression::CallExpression(CallExpression {
                    callee: Box::new(member(Expression::Super, "suara")),
                    arguments: vec![],
                    line: 2,
                }))])
            ),
            other => panic!("{:?}", other),
        }
        // also inside a text
        assert!(parse_program("A() { m() { tulis(\"{..x}\") } }").is_ok());
        // outside a class there is no parent either
        assert_eq!(
            err("..x"),
            "baris 1, kolom 1: `.nama` hanya bisa dipakai di dalam kelas, sebagai objek yang sedang dijalankan"
        );
        // two dots with a gap between them are not it
        assert_eq!(
            err("Hewan() { m() { . .x } }"),
            "baris 1, kolom 19: diharapkan nama properti, ditemukan `.`"
        );
        assert_eq!(
            err("Hewan() { m() { ...x } }"),
            "baris 1, kolom 19: diharapkan nama properti, ditemukan `.`"
        );
    }

    #[test]
    fn class_errors() {
        assert_eq!(
            err("Hewan() { tunggu f() }"),
            "baris 1, kolom 11: `tunggu` tidak bisa dipakai langsung di dalam kelas, pakai di dalam metode"
        );
        assert_eq!(
            err("Kucing() turunan Hewan(tunggu f()) { }"),
            "baris 1, kolom 24: `tunggu` tidak bisa dipakai langsung di dalam kelas, pakai di dalam metode"
        );
        // the parent is built before the object exists, so there is no `.nama` yet
        assert_eq!(
            err("Kucing() turunan Hewan(.x) { }"),
            "baris 1, kolom 24: `.nama` hanya bisa dipakai di dalam kelas, sebagai objek yang sedang dijalankan"
        );
        assert_eq!(
            err("Kucing() turunan Hewan {\n}"),
            "baris 1, kolom 24: diharapkan `(`, ditemukan `{`"
        );
        assert_eq!(err("Hewan() {"), "baris 1, kolom 10: diharapkan `}`, ditemukan akhir kode");
    }

    #[test]
    fn statements_end_at_a_line_break() {
        assert_eq!(ok("misal x = 1\nmisal y = 2"), ok("misal x = 1; misal y = 2;"));
        assert_eq!(ok("tulis(1)\ntulis(2)").len(), 2);
        // before `}` and at the end of the code
        assert_eq!(ok("jika x { berhenti }").len(), 1);
        assert_eq!(ok("fungsi f() { hasilkan 1 }").len(), 1);
        // `;` is still fine, and so is a statement after it on the same line
        assert_eq!(ok("misal a = 1; misal b = 2").len(), 2);
        assert_eq!(
            err("misal x = 1 2"),
            "baris 1, kolom 13: diharapkan `;` atau baris baru, ditemukan `2`"
        );
    }

    #[test]
    fn a_token_that_starts_a_line_does_not_continue_the_expression() {
        // `[`, `(` and `-` begin a new statement
        assert_eq!(ok("misal x = a\n[1].c").len(), 2);
        assert_eq!(ok("f\n(1)").len(), 2);
        assert_eq!(ok("a\n- b").len(), 2);
        // a binary operator has to stay on the line of its left operand
        assert_eq!(
            err("misal x = a\n+ b"),
            "baris 2, kolom 1: ekspresi tidak lengkap, ditemukan `+`"
        );
        assert_eq!(err("misal x = a\ndan b"), "baris 2, kolom 1: `dan` adalah kata kunci dan tidak bisa dipakai sebagai nama");
    }

    #[test]
    fn an_operator_at_the_end_of_a_line_continues() {
        assert_eq!(ok("misal x = 1 +\n  2"), ok("misal x = 1 + 2;"));
        assert_eq!(ok("misal x = a dan\n  b"), ok("misal x = a dan b;"));
        assert_eq!(ok("misal x = xs.\n  panjang"), ok("misal x = xs.panjang;"));
    }

    #[test]
    fn line_breaks_do_not_matter_inside_brackets() {
        assert_eq!(ok("misal x = [1\n  + 2]"), ok("misal x = [1 + 2];"));
        assert_eq!(ok("f(1\n, 2\n)"), ok("f(1, 2);"));
        assert_eq!(ok("misal x = (1\n  + 2)"), ok("misal x = (1 + 2);"));
        // ... but a block inside brackets goes back to one statement per line
        assert_eq!(
            ok("f(fungsi () {\n  misal a = 1\n  misal b = 2\n})").len(),
            1
        );
        assert_eq!(
            err("f(fungsi () {\n  misal a = 1 misal b = 2\n})"),
            "baris 2, kolom 15: diharapkan `;` atau baris baru, ditemukan `misal`"
        );
    }

    #[test]
    fn hasilkan_without_a_value_ends_at_the_line() {
        assert_eq!(
            ok("fungsi f() {\n  hasilkan\n  x\n}"),
            vec![Statement::FunctionDeclaration(FunctionDeclaration {
                id: id("f"),
                params: vec![],
                body: block(vec![Statement::Return(None), Statement::Expression(ident("x"))]),
                is_async: false,
            })]
        );
    }

    #[test]
    fn blocks_that_make_a_value() {
        assert_eq!(
            ok("misal v = { misal x = 5; hasilkan x + 1 }"),
            vec![let_(
                "v",
                block_value(
                    vec![
                        let_("x", num(5.0)),
                        Statement::Return(Some(bin(ident("x"), Operator::Addition, num(1.0)))),
                    ],
                    false
                )
            )]
        );
        // `{ nama: ... }`, `{ "k": ... }` and `{}` are objects, anything else is a block
        assert!(matches!(
            &ok("misal v = { a: 1 }")[0],
            Statement::VariableDeclaration(VariableDeclaration { value: Expression::Object(_), .. })
        ));
        assert!(matches!(
            &ok("misal v = {}")[0],
            Statement::VariableDeclaration(VariableDeclaration { value: Expression::Object(_), .. })
        ));
        assert!(matches!(
            &ok("misal v = { a }")[0],
            Statement::VariableDeclaration(VariableDeclaration { value: Expression::Block(_), .. })
        ));
    }

    #[test]
    fn a_block_that_uses_tunggu_is_async_and_so_is_its_function() {
        match &ok("fungsi g() { misal v = { hasilkan tunggu f() } }")[0] {
            Statement::FunctionDeclaration(g) => {
                assert!(g.is_async);
                match &g.body.body.as_ref().unwrap()[0] {
                    Statement::VariableDeclaration(VariableDeclaration {
                        value: Expression::Block(b),
                        ..
                    }) => assert!(b.is_async),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn errors_have_positions() {
        assert_eq!(err("misal x = (1"), "baris 1, kolom 13: diharapkan `)`, ditemukan akhir kode");
        assert_eq!(err("misal x = 1;\nmisal y = ;"), "baris 2, kolom 11: ekspresi tidak lengkap, ditemukan `;`");
        assert_eq!(err("misal jika = 1;"), "baris 1, kolom 7: `jika` adalah kata kunci dan tidak bisa dipakai sebagai nama");
        assert_eq!(err("jika x {"), "baris 1, kolom 9: diharapkan `}`, ditemukan akhir kode");
        assert_eq!(err("misal s = \"abc;"), "baris 1, kolom 11: teks tidak ditutup dengan `\"`");
        assert_eq!(err("misal x = 2x;"), "baris 1, kolom 11: angka tidak valid");
        assert_eq!(err("misal x = 1 @ 2;"), "baris 1, kolom 13: karakter `@` tidak dikenal");
    }
}
