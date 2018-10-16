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

#[derive(PartialEq, Debug)]
pub struct IfStatement {
    pub test: Expression,
    pub consequent: BlockStatement,
    // TODO alternate: Option(AlternateStatement)
}

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
    IfStatement(IfStatement),
}

#[derive(PartialEq, Debug)]
pub struct Program {
    pub body: Vec<Statement>,
}
