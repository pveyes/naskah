#[derive(PartialEq, Debug)]
pub enum LiteralValue {
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
pub struct VariableDeclaration {
  pub id: Identifier,
  pub value: LiteralValue,
}

// #[derive(PartialEq, Debug)]
// pub enum Operator {
//   Equal,
//   NotEqual,
//   GreaterThan,
//   LessThan,
// }

// #[derive(PartialEq, Debug)]
// pub enum Operand {
//   Identifier(Identifier),
//   Literal(LiteralValue),
// }

// #[derive(PartialEq, Debug)]
// pub struct BinaryExpression {
//   pub left: Operand,
//   pub right: Operand,
//   pub operator: Operator,
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
