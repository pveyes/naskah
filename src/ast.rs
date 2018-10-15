#[derive(PartialEq, Debug)]
pub enum Literal {
    Number(i64),
    Null,
    String(String),
    Boolean(bool),
}

#[derive(PartialEq, Debug)]
pub struct Identifier {
    pub name: String,
}

#[derive(PartialEq, Debug)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    BinaryExpression(Box<BinaryExpression>),
}

#[derive(PartialEq, Debug)]
pub enum Operator {
    Plus,
    Minus,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
}

#[derive(PartialEq, Debug)]
pub struct BinaryExpression {
    pub left: Expression,
    pub right: Expression,
    pub operator: Operator,
}

#[derive(PartialEq, Debug)]
pub struct VariableDeclaration {
    pub id: Identifier,
    pub value: Expression,
}

// #[derive(PartialEq, Debug)]
// pub enum Operand {
//   Identifier(Identifier),
//   Literal(LiteralValue),
// }

// #[derive(PartialEq, Debug)]
// pub struct IFStatement {
//   pub test: BinaryExpression,
// }

#[derive(PartialEq, Debug)]
pub enum ProgramBody {
    VariableDeclaration(VariableDeclaration),
    // IFStatement(IFStatement),
}

#[derive(PartialEq, Debug)]
pub struct Program {
    pub body: Vec<ProgramBody>,
}
