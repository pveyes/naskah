#[macro_use]
extern crate nom;
extern crate lazy_static;
extern crate regex;

mod ast;
mod parser;

use ast::Identifier;
use ast::Literal;
use ast::Program;
use ast::ProgramBody;
use ast::VariableDeclaration;
use ast::VariableValue;
use parser::parse;

fn main() {
    let d1 = parse("misal x = 1 ;").unwrap();

    // var x = 1;
    let d2 = VariableDeclaration {
        id: Identifier {
            name: String::from("x"),
        },
        value: VariableValue::Literal(Literal::Number(1)),
    };

    let pg1 = Program {
        body: vec![ProgramBody::VariableDeclaration(d1)],
    };
    let pg2 = Program {
        body: vec![ProgramBody::VariableDeclaration(d2)],
    };
    assert_eq!(pg1, pg2);
}
