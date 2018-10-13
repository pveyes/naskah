#[macro_use]
extern crate nom;
extern crate byteorder;

mod ast;
mod parser;

use ast::Identifier;
use ast::LiteralValue;
use ast::Program;
use ast::ProgramBody;
use ast::VariableDeclaration;

fn main() {
    // let d1 = parser.parse("misal x = 1");

    // var x = 1
    let d1 = VariableDeclaration {
        id: Identifier {
            name: String::from("x"),
        },
        value: LiteralValue::Number(1),
    };

    // var x = 2;
    let d2 = VariableDeclaration {
        id: Identifier {
            name: String::from("x"),
        },
        value: LiteralValue::Number(2),
    };

    let pg1 = Program {
        body: vec![ProgramBody::VariableDeclaration(d1)],
    };
    let pg2 = Program {
        body: vec![ProgramBody::VariableDeclaration(d2)],
    };
    assert_eq!(pg1, pg2);
}
