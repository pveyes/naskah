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
    let mut res = String::new();

    let val = *b;
    let left = print_expression(val.left);
    let right = print_expression(val.right);
    let operator = print_operator(val.operator);

    res.push_str(&left);
    res.push_str(" ");
    res.push_str(&operator);
    res.push_str(" ");
    res.push_str(&right);
    res
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
    }
}

fn print_call_expression(c: CallExpression) -> String {
    let mut x = String::new();
    x.push_str(&c.callee.name);
    x.push_str("(");
    for argument in c.arguments {
        let arg = print_expression(argument);
        x.push_str(&arg);
        x.push_str(",");
    }
    x.push_str(")");
    x
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
    let id = print_identifier(v.id);
    let val = print_expression(v.value);
    let mut st = String::new();
    st.push_str(&insert_indent(depth));
    st.push_str("var ");
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
    res.push_str("while(true) ");
    res.push_str(&print_block_statement(b, depth));
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
                id: Identifier {
                    name: String::from("x"),
                },
                value: Expression::Literal(Literal::Null),
            })],
        });

        assert_eq!(&s, &"var x = null;\n")
    }
}
