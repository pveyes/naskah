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
pub struct BlockStatement {
    pub body: Vec<Statement>,
}

#[derive(PartialEq, Debug)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    BlockStatement(BlockStatement),
    Empty,
    Loop(BlockStatement),
    // IFStatement(IFStatement)
    // LoopStatement(LoopStatement)
}

#[derive(PartialEq, Debug)]
pub struct Program {
    pub body: Vec<Statement>,
}
