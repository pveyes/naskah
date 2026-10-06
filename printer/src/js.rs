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
        Expression::UnaryExpression(_) => PREC_UNARY,
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
        other => other,
    }
}

fn print_call_expression(c: CallExpression) -> String {
    let arguments: Vec<String> = c.arguments.into_iter().map(print_expression).collect();
    format!("{}({})", builtin(&c.callee.name), arguments.join(", "))
}

fn print_assignment_expression(s: AssignmentExpression) -> String {
    let mut res = String::new();
    res.push_str(&print_identifier(s.id));
    res.push_str(" = ");
    res.push_str(&print_expression(*s.value));
    res
}

fn print_expression(e: Expression) -> String {
    match e {
        Expression::Assignment(e) => print_assignment_expression(e),
        Expression::Literal(l) => print_literal(l),
        Expression::BinaryExpression(b) => print_binary_expression(b),
        Expression::UnaryExpression(u) => print_unary_expression(u),
        Expression::CallExpression(c) => print_call_expression(c),
        Expression::Identifier(i) => print_identifier(i),
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
    let val = print_expression(v.value);
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
    res.push_str(&print_expression(i.test));
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
    res.push_str(&print_expression(w.test));
    res.push_str(") ");
    res.push_str(&print_block_statement(w.body, depth));
    res
}

fn print_function_declaration(f: FunctionDeclaration, depth: u8) -> String {
    let params: Vec<String> = f.params.into_iter().map(print_identifier).collect();
    let mut res = insert_indent(depth);
    res.push_str(&format!("function {}({}) ", f.id.name, params.join(", ")));
    res.push_str(&print_block_statement(f.body, depth));
    res
}

fn print_return_statement(value: Option<Expression>, depth: u8) -> String {
    let mut res = insert_indent(depth);
    res.push_str("return");
    if let Some(e) = value {
        res.push_str(" ");
        res.push_str(&print_expression(e));
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
            res.push_str(&print_expression(e));
            res.push_str(";");
            res
        }
        Statement::VariableDeclaration(v) => print_variable_declaration(v, depth),
        Statement::BlockStatement(s) => print_block_statement(s, depth),
        Statement::IfStatement(s) => print_if_statement(s, depth, false),
        Statement::Loop(s) => print_loop_statement(s, depth),
        Statement::While(s) => print_while_statement(s, depth),
        Statement::FunctionDeclaration(f) => print_function_declaration(f, depth),
        Statement::Return(e) => print_return_statement(e, depth),
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
