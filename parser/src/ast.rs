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
pub struct CallExpression {
    pub callee: Identifier,
    pub arguments: Vec<Expression>,
}

#[derive(PartialEq, Debug)]
pub struct AssignmentExpression {
    pub id: Identifier,
    pub value: Box<Expression>,
}

#[derive(PartialEq, Debug)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    BinaryExpression(Box<BinaryExpression>),
    CallExpression(CallExpression),
    Assignment(AssignmentExpression),
}

#[derive(PartialEq, Debug)]
pub enum Operator {
    Addition,
    Substraction,
    Multiplication,
    Division,
    Remainder,
    Exponentiation,
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
pub enum AlternateStatement {
    IfStatement(Box<IfStatement>),
    BlockStatement(BlockStatement),
}

#[derive(PartialEq, Debug)]
pub struct IfStatement {
    pub test: Expression,
    pub consequent: BlockStatement,
    pub alternate: Option<AlternateStatement>,
}

#[derive(PartialEq, Debug)]
pub struct BlockStatement {
    pub body: Option<Vec<Statement>>,
}

#[derive(PartialEq, Debug)]
pub enum Statement {
    Break,
    Continue,
    Expression(Expression),
    VariableDeclaration(VariableDeclaration),
    BlockStatement(BlockStatement),
    Loop(BlockStatement),
    IfStatement(IfStatement),
}

#[derive(PartialEq, Debug)]
pub struct Program {
    pub body: Vec<Statement>,
}
