use parser::ast::*;

fn insert_indent(depth: u8) -> String {
    let mut res = String::new();
    let mut x = 0;

    if depth == 0 {
        return res;
    }

    loop {
        res.push_str("  ");
        x = x + 1;
        if x >= depth {
            break;
        }
    }

    res
}

fn print_literal(l: Literal) -> String {
    match l {
        Literal::Null => String::from("null"),
        Literal::Boolean(bool) => match bool {
            true => String::from("true"),
            false => String::from("false"),
        },
        Literal::Number(n) => n.to_string(),
        Literal::String(s) => {
            let mut x = String::new();
            x.push_str("\"");
            x.push_str(&s);
            x.push_str("\"");
            x
        }
    }
}

fn print_identifier(i: Identifier) -> String {
    i.name
}

fn print_binary_expression(b: Box<BinaryExpression>) -> String {
    let val = *b;
    let prec = operator_precedence(&val.operator);
    let right_assoc = val.operator == Operator::Exponentiation;

    let left_prec = precedence(&val.left);
    let right_prec = precedence(&val.right);
    // JS rejects a unary operand on the left of `**`: (-a) ** b
    let left_parens = left_prec < prec
        || (left_prec == prec && right_assoc)
        || (right_assoc && left_prec == PREC_UNARY);
    let right_parens = right_prec < prec || (right_prec == prec && !right_assoc);

    let operator = print_operator(val.operator);
    let left = print_operand(val.left, left_parens);
    let right = print_operand(val.right, right_parens);

    format!("{} {} {}", left, operator, right)
}

fn print_unary_expression(u: Box<UnaryExpression>) -> String {
    let val = *u;
    let symbol = match val.operator {
        UnaryOperator::Not => "!",
        UnaryOperator::Negate => "-",
    };
    // avoid printing `--x`, which JS reads as a decrement
    let nested_negation = val.operator == UnaryOperator::Negate
        && match &val.argument {
            Expression::UnaryExpression(inner) => inner.operator == UnaryOperator::Negate,
            _ => false,
        };
    let parens = precedence(&val.argument) < PREC_UNARY || nested_negation;
    format!("{}{}", symbol, print_operand(val.argument, parens))
}

fn print_operator(op: Operator) -> String {
    match op {
        Operator::Addition => String::from("+"),
        Operator::Substraction => String::from("-"),
        Operator::Multiplication => String::from("*"),
        Operator::Division => String::from("/"),
        Operator::Remainder => String::from("%"),
        Operator::Exponentiation => String::from("**"),
        Operator::Equal => String::from("==="),
        Operator::NotEqual => String::from("!=="),
        Operator::GreaterThan => String::from(">"),
        Operator::LessThan => String::from("<"),
        Operator::GreaterThanOrEqualTo => String::from(">="),
        Operator::LessThanOrEqualTo => String::from("<="),
        Operator::And => String::from("&&"),
        Operator::Or => String::from("||"),
    }
}

// Same ordering as JavaScript, so a tree built from Naskah precedence
// prints with exactly the parentheses JS needs.
const PREC_ASSIGNMENT: u8 = 0;
const PREC_UNARY: u8 = 8;
const PREC_ATOM: u8 = 9;

fn operator_precedence(op: &Operator) -> u8 {
    match op {
        Operator::Or => 1,
        Operator::And => 2,
        Operator::Equal | Operator::NotEqual => 3,
        Operator::GreaterThan
        | Operator::LessThan
        | Operator::GreaterThanOrEqualTo
        | Operator::LessThanOrEqualTo => 4,
        Operator::Addition | Operator::Substraction => 5,
        Operator::Multiplication | Operator::Division | Operator::Remainder => 6,
        Operator::Exponentiation => 7,
    }
}

fn precedence(e: &Expression) -> u8 {
    match e {
        Expression::Assignment(_) => PREC_ASSIGNMENT,
        Expression::BinaryExpression(b) => operator_precedence(&b.operator),
        Expression::UnaryExpression(_) | Expression::Await(_) => PREC_UNARY,
        _ => PREC_ATOM,
    }
}

/// Print `e` as an operand, wrapping it in parentheses when `needs_parens`.
fn print_operand(e: Expression, needs_parens: bool) -> String {
    let s = print_expression(e);
    if needs_parens {
        format!("({})", s)
    } else {
        s
    }
}

/// Naskah built-ins and the JavaScript they stand for.
fn builtin(name: &str) -> &str {
    match name {
        "tulis" => "console.log",
        "tanya" => "prompt",
        "Galat" => "Error",
        other => other,
    }
}

/// Names that mean something else when used as a value rather than called.
fn builtin_value(name: &str) -> &str {
    match name {
        "Galat" => "Error",
        other => other,
    }
}

/// Property and method names, applied to `a.name` and to `{ name: ... }` alike
/// so a name means the same thing wherever it is used.
fn property_name(name: &str) -> &str {
    match name {
        "panjang" => "length",
        "tambah" => "push",
        "gabung" => "join",
        "balik" => "reverse",
        "peta" => "map",
        "saring" => "filter",
        "cari" => "find",
        "urut" => "sort",
        "pesan" => "message",
        other => other,
    }
}

/// Brackets a Naskah line number in front of a printed `tulis` call. `print`
/// leaves them in; `extract_call_sites` turns them into a source map.
pub const SITE_START: char = '\u{E000}';
pub const SITE_END: char = '\u{E001}';

fn print_call_expression(c: CallExpression) -> String {
    // tunda(ms) is a sleep: a promise that resolves after `ms` milliseconds
    if c.arguments.len() == 1 && matches!(&*c.callee, Expression::Identifier(i) if i.name == "tunda") {
        let ms = print_expression(c.arguments.into_iter().next().unwrap());
        return format!("new Promise((resolve) => setTimeout(resolve, {}))", ms);
    }

    let callee = match *c.callee {
        Expression::Identifier(i) if i.name == "tulis" => {
            format!("{}{}{}{}", SITE_START, c.line, SITE_END, builtin(&i.name))
        }
        Expression::Identifier(i) => builtin(&i.name).to_string(),
        other => {
            let parens = precedence(&other) < PREC_ATOM;
            print_operand(other, parens)
        }
    };
    let arguments: Vec<String> = c.arguments.into_iter().map(print_expression).collect();
    format!("{}({})", callee, arguments.join(", "))
}

/// An operand of `.name`, `[i]`: `(a + b).x` and `(5).x` need their parentheses.
fn print_object_operand(e: Expression) -> String {
    let parens = precedence(&e) < PREC_ATOM || matches!(e, Expression::Literal(Literal::Number(_)));
    print_operand(e, parens)
}

fn print_member_expression(m: Box<MemberExpression>) -> String {
    let m = *m;
    format!("{}.{}", print_object_operand(m.object), property_name(&m.property))
}

fn print_index_expression(i: Box<IndexExpression>) -> String {
    let i = *i;
    format!("{}[{}]", print_object_operand(i.object), print_expression(i.index))
}

fn print_list(items: Vec<Expression>) -> String {
    let items: Vec<String> = items.into_iter().map(print_expression).collect();
    format!("[{}]", items.join(", "))
}

fn print_object(properties: Vec<Property>) -> String {
    if properties.is_empty() {
        return String::from("{}");
    }
    let properties: Vec<String> = properties
        .into_iter()
        .map(|p| {
            let key = match p.key {
                PropertyKey::Name(name) => property_name(&name).to_string(),
                PropertyKey::Text(text) => format!("\"{}\"", text),
            };
            format!("{}: {}", key, print_expression(p.value))
        })
        .collect();
    format!("{{ {} }}", properties.join(", "))
}

fn print_assignment_expression(s: AssignmentExpression) -> String {
    format!("{} = {}", print_expression(*s.target), print_expression(*s.value))
}

fn print_new_expression(n: Box<NewExpression>) -> String {
    let n = *n;
    let callee = match n.callee {
        Expression::Identifier(i) => builtin(&i.name).to_string(),
        other => print_expression(other),
    };
    let arguments: Vec<String> = n.arguments.into_iter().map(print_expression).collect();
    format!("new {}({})", callee, arguments.join(", "))
}

fn print_await_expression(argument: Box<Expression>) -> String {
    let parens = precedence(&argument) < PREC_UNARY;
    format!("await {}", print_operand(*argument, parens))
}

fn print_params(params: Vec<Identifier>) -> String {
    let names: Vec<String> = params.into_iter().map(print_identifier).collect();
    names.join(", ")
}

/// A function value's body is printed with no indentation; `expr_at` lines it
/// up with the statement it ends up in.
fn print_function_expression(f: Box<FunctionExpression>) -> String {
    let f = *f;
    format!(
        "{}function ({}) {}",
        if f.is_async { "async " } else { "" },
        print_params(f.params),
        print_block_statement(f.body, 0)
    )
}

/// Text as it appears inside a JS template literal.
fn template_text(raw: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;

    // `$` right before a `{` would start a JS interpolation, and a backtick would end the literal
    fn push(out: &mut String, c: char) {
        let trailing_backslashes = out
            .strip_suffix('$')
            .map(|rest| rest.chars().rev().take_while(|c| *c == '\\').count());
        match c {
            '`' => out.push_str("\\`"),
            '{' if trailing_backslashes.map_or(false, |n| n % 2 == 0) => {
                out.pop();
                out.push_str("\\${");
            }
            _ => out.push(c),
        }
    }

    while i < chars.len() {
        match (chars[i], chars.get(i + 1)) {
            // `\{` is how Naskah writes a literal brace
            ('\\', Some('{')) => {
                push(&mut out, '{');
                i += 2;
            }
            ('\\', Some(next)) => {
                out.push('\\');
                out.push(*next);
                i += 2;
            }
            (c, _) => {
                push(&mut out, c);
                i += 1;
            }
        }
    }
    out
}

fn print_template(parts: Vec<TemplatePart>) -> String {
    let mut out = String::from("`");
    for part in parts {
        match part {
            TemplatePart::Text(text) => out.push_str(&template_text(&text)),
            TemplatePart::Expression(e) => {
                out.push_str("${");
                out.push_str(&print_expression(e));
                out.push('}');
            }
        }
    }
    out.push('`');
    out
}

/// An expression inside a statement at `depth`: lines after the first (only
/// function bodies have any) are indented to match the statement.
fn expr_at(e: Expression, depth: u8) -> String {
    let printed = print_expression(e);
    if depth == 0 || !printed.contains('\n') {
        return printed;
    }
    let indent = insert_indent(depth);
    printed
        .split('\n')
        .enumerate()
        .map(|(i, line)| if i == 0 { line.to_string() } else { format!("{}{}", indent, line) })
        .collect::<Vec<_>>()
        .join("\n")
}

fn print_expression(e: Expression) -> String {
    match e {
        Expression::Assignment(e) => print_assignment_expression(e),
        Expression::Literal(l) => print_literal(l),
        Expression::BinaryExpression(b) => print_binary_expression(b),
        Expression::UnaryExpression(u) => print_unary_expression(u),
        Expression::CallExpression(c) => print_call_expression(c),
        Expression::Identifier(i) => builtin_value(&i.name).to_string(),
        Expression::Member(m) => print_member_expression(m),
        Expression::Index(i) => print_index_expression(i),
        Expression::List(items) => print_list(items),
        Expression::Object(properties) => print_object(properties),
        Expression::Template(parts) => print_template(parts),
        Expression::Function(f) => print_function_expression(f),
        Expression::New(n) => print_new_expression(n),
        Expression::Await(a) => print_await_expression(a),
        Expression::This => String::from("this"),
        Expression::Super => String::from("super"),
    }
}

fn print_block_statement(b: BlockStatement, depth: u8) -> String {
    let mut res = String::new();
    res.push_str("{\n");
    let content = match b.body {
        Some(statements) => {
            let mut sts = String::new();
            for statement in statements {
                sts.push_str(&print_statement(statement, depth + 1));
            }
            sts
        }
        None => String::from(""),
    };
    res.push_str(&content);
    res.push_str(&insert_indent(depth));
    res.push_str("}");
    res
}

fn print_variable_declaration(v: VariableDeclaration, depth: u8) -> String {
    let kind = match v.kind {
        VariableKind::Let => "let ",
        VariableKind::Const => "const ",
    };
    let id = print_identifier(v.id);
    let val = expr_at(v.value, depth);
    let mut st = String::new();
    st.push_str(&insert_indent(depth));
    st.push_str(kind);
    st.push_str(&id);
    st.push_str(" = ");
    st.push_str(&val);
    st.push_str(";");
    st
}

fn print_if_statement(i: IfStatement, depth: u8, inside_else: bool) -> String {
    let mut res = String::new();
    let else_statement = match i.alternate {
        Some(st) => {
            let mut res = String::new();
            let x = match st {
                AlternateStatement::IfStatement(i) => print_if_statement(*i, depth, true),
                AlternateStatement::BlockStatement(b) => print_block_statement(b, depth),
            };

            res.push_str(" else ");
            res.push_str(&x);
            res
        }
        None => String::from(""),
    };

    if !inside_else {
        res.push_str(&insert_indent(depth));
    }
    res.push_str("if (");
    res.push_str(&expr_at(i.test, depth));
    res.push_str(") ");
    res.push_str(&print_block_statement(i.consequent, depth));
    res.push_str(&else_statement);
    res
}

fn print_loop_statement(b: BlockStatement, depth: u8) -> String {
    let mut res = String::new();
    res.push_str(&insert_indent(depth));
    res.push_str("while (true) ");
    res.push_str(&print_block_statement(b, depth));
    res
}

fn print_while_statement(w: WhileStatement, depth: u8) -> String {
    let mut res = insert_indent(depth);
    res.push_str("while (");
    res.push_str(&expr_at(w.test, depth));
    res.push_str(") ");
    res.push_str(&print_block_statement(w.body, depth));
    res
}

fn print_for_range_statement(f: ForRangeStatement, depth: u8) -> String {
    let descending = match &f.step {
        Some(Expression::UnaryExpression(u)) => u.operator == UnaryOperator::Negate,
        _ => false,
    };
    let var = f.var.name;
    let update = match f.step {
        Some(step) => format!("{} += {}", var, expr_at(step, depth)),
        None => format!("{}++", var),
    };
    format!(
        "{}for (let {} = {}; {} {} {}; {}) {}",
        insert_indent(depth),
        var,
        expr_at(f.from, depth),
        var,
        if descending { ">=" } else { "<=" },
        expr_at(f.to, depth),
        update,
        print_block_statement(f.body, depth)
    )
}

fn print_for_each_statement(f: ForEachStatement, depth: u8) -> String {
    format!(
        "{}for (let {} of {}) {}",
        insert_indent(depth),
        f.var.name,
        expr_at(f.iterable, depth),
        print_block_statement(f.body, depth)
    )
}

/// `pilih` becomes an if/else chain rather than a JS `switch`, so `berhenti;`
/// and `lanjut;` inside a `kalau` still apply to the loop around it.
fn print_switch_statement(s: SwitchStatement, depth: u8) -> String {
    let (temp, subject) = match s.discriminant {
        Expression::Identifier(i) => (None, Expression::Identifier(i)),
        other => {
            let name = Identifier { name: format!("_pilih{}", depth) };
            (Some((name.clone(), other)), Expression::Identifier(name))
        }
    };

    let mut alternate = s.default.map(AlternateStatement::BlockStatement);
    for case in s.cases.into_iter().rev() {
        let mut matches = case.tests.into_iter().map(|t| {
            Expression::BinaryExpression(Box::new(BinaryExpression {
                left: subject.clone(),
                right: t,
                operator: Operator::Equal,
            }))
        });
        let first = matches.next().unwrap();
        let test = matches.fold(first, |left, right| {
            Expression::BinaryExpression(Box::new(BinaryExpression {
                left,
                right,
                operator: Operator::Or,
            }))
        });
        alternate = Some(AlternateStatement::IfStatement(Box::new(IfStatement {
            test,
            consequent: case.body,
            alternate,
        })));
    }
    let chain = match alternate {
        Some(AlternateStatement::IfStatement(i)) => *i,
        _ => unreachable!("the parser requires at least one kalau"),
    };

    match temp {
        None => print_if_statement(chain, depth, false),
        Some((name, value)) => {
            let wrapper = BlockStatement {
                body: Some(vec![
                    Statement::VariableDeclaration(VariableDeclaration {
                        kind: VariableKind::Const,
                        id: name,
                        value,
                    }),
                    Statement::IfStatement(chain),
                ]),
            };
            insert_indent(depth) + &print_block_statement(wrapper, depth)
        }
    }
}

fn print_function_declaration(f: FunctionDeclaration, depth: u8) -> String {
    let mut res = insert_indent(depth);
    res.push_str(&format!(
        "{}function {}({}) ",
        if f.is_async { "async " } else { "" },
        f.id.name,
        print_params(f.params)
    ));
    res.push_str(&print_block_statement(f.body, depth));
    res
}

fn print_class_declaration(c: ClassDeclaration, depth: u8) -> String {
    let inner = insert_indent(depth + 1);
    let mut res = insert_indent(depth);
    res.push_str(&format!("class {}", c.id.name));
    if let Some(parent) = c.parent {
        res.push_str(&format!(" extends {}", parent.name));
    }
    res.push_str(" {\n");

    if let Some(ctor) = c.constructor {
        res.push_str(&format!(
            "{}constructor({}) {}\n",
            inner,
            print_params(ctor.params),
            print_block_statement(ctor.body, depth + 1)
        ));
    }
    for m in c.methods {
        res.push_str(&format!(
            "{}{}{}({}) {}\n",
            inner,
            if m.is_async { "async " } else { "" },
            property_name(&m.name),
            print_params(m.params),
            print_block_statement(m.body, depth + 1)
        ));
    }

    res.push_str(&insert_indent(depth));
    res.push_str("}");
    res
}

fn print_try_statement(t: TryStatement, depth: u8) -> String {
    let mut res = insert_indent(depth);
    res.push_str("try ");
    res.push_str(&print_block_statement(t.block, depth));
    if let Some(handler) = t.handler {
        match handler.param {
            Some(param) => res.push_str(&format!(" catch ({}) ", param.name)),
            None => res.push_str(" catch "),
        }
        res.push_str(&print_block_statement(handler.body, depth));
    }
    if let Some(finalizer) = t.finalizer {
        res.push_str(" finally ");
        res.push_str(&print_block_statement(finalizer, depth));
    }
    res
}

fn print_return_statement(value: Option<Expression>, depth: u8) -> String {
    let mut res = insert_indent(depth);
    res.push_str("return");
    if let Some(e) = value {
        res.push_str(" ");
        res.push_str(&expr_at(e, depth));
    }
    res.push_str(";");
    res
}

fn print_statement(s: Statement, depth: u8) -> String {
    let mut res = String::new();
    let x: String = match s {
        Statement::Expression(e) => {
            let mut res = String::new();
            res.push_str(&insert_indent(depth));
            let printed = expr_at(e, depth);
            if printed.starts_with('{') {
                // an object literal here would be read as a block
                res.push_str(&format!("({})", printed));
            } else {
                res.push_str(&printed);
            }
            res.push_str(";");
            res
        }
        Statement::VariableDeclaration(v) => print_variable_declaration(v, depth),
        Statement::BlockStatement(s) => insert_indent(depth) + &print_block_statement(s, depth),
        Statement::IfStatement(s) => print_if_statement(s, depth, false),
        Statement::Loop(s) => print_loop_statement(s, depth),
        Statement::While(s) => print_while_statement(s, depth),
        Statement::ForRange(s) => print_for_range_statement(s, depth),
        Statement::ForEach(s) => print_for_each_statement(s, depth),
        Statement::Switch(s) => print_switch_statement(s, depth),
        Statement::FunctionDeclaration(f) => print_function_declaration(f, depth),
        Statement::Return(e) => print_return_statement(e, depth),
        Statement::ClassDeclaration(c) => print_class_declaration(c, depth),
        Statement::Try(t) => print_try_statement(t, depth),
        Statement::Throw(e) => format!("{}throw {};", insert_indent(depth), expr_at(e, depth)),
        Statement::Break => insert_indent(depth) + &String::from("break;"),
        Statement::Continue => insert_indent(depth) + &String::from("continue;"),
    };
    res.push_str(&x);
    res.push_str("\n");
    res
}

pub fn print(ast: Program) -> String {
    let mut js = String::new();
    for statement in ast.body {
        js.push_str(&print_statement(statement, 0));
    }

    js
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn name() {
        let s = print(Program {
            body: vec![Statement::VariableDeclaration(VariableDeclaration {
                kind: VariableKind::Let,
                id: Identifier {
                    name: String::from("x"),
                },
                value: Expression::Literal(Literal::Null),
            })],
        });

        assert_eq!(&s, &"let x = null;\n")
    }
}
