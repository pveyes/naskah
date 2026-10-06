#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    Number(f64),
    Null,
    String(String),
    Boolean(bool),
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Identifier {
    pub name: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    /// Source line (1-based) of the opening parenthesis.
    pub line: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AssignmentExpression {
    /// An identifier, member or index expression.
    pub target: Box<Expression>,
    pub value: Box<Expression>,
}

/// `object.property`
#[derive(PartialEq, Debug, Clone)]
pub struct MemberExpression {
    pub object: Expression,
    pub property: String,
}

/// `object[index]`
#[derive(PartialEq, Debug, Clone)]
pub struct IndexExpression {
    pub object: Expression,
    pub index: Expression,
}

#[derive(PartialEq, Debug, Clone)]
pub enum PropertyKey {
    /// `nama: ...`
    Name(String),
    /// `"nama lengkap": ...`, raw text between the quotes
    Text(String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Property {
    pub key: PropertyKey,
    pub value: Expression,
}

#[derive(PartialEq, Debug, Clone)]
pub enum UnaryOperator {
    /// `bukan`
    Not,
    /// `-`
    Negate,
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub argument: Expression,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    BinaryExpression(Box<BinaryExpression>),
    UnaryExpression(Box<UnaryExpression>),
    CallExpression(CallExpression),
    Assignment(AssignmentExpression),
    Member(Box<MemberExpression>),
    Index(Box<IndexExpression>),
    List(Vec<Expression>),
    Object(Vec<Property>),
    /// `"Halo, {nama}!"`
    Template(Vec<TemplatePart>),
    /// `fungsi (x) { }`, async when its body uses `tunggu`
    Function(Box<FunctionExpression>),
    /// `Kelas(argumen)`: calling a capitalised name builds an object
    New(Box<NewExpression>),
    /// `tunggu ekspresi`
    Await(Box<Expression>),
    /// `{ misal x = 5; hasilkan x + 1 }`, a block that produces a value
    Block(Box<BlockExpression>),
    /// Only appears as `.nama`, the object a class method is running on.
    This,
    /// Only appears as `..nama`, the parent class's version of `.nama`.
    Super,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TemplatePart {
    /// Raw text, escape sequences left as written.
    Text(String),
    Expression(Expression),
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionExpression {
    pub params: Vec<Identifier>,
    pub body: BlockStatement,
    pub is_async: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct BlockExpression {
    pub body: BlockStatement,
    /// True when the block uses `tunggu`.
    pub is_async: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct NewExpression {
    pub callee: Expression,
    pub arguments: Vec<Expression>,
}

#[derive(PartialEq, Debug, Clone)]
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
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    /// `dan`
    And,
    /// `atau`
    Or,
}

#[derive(PartialEq, Debug, Clone)]
pub struct BinaryExpression {
    pub left: Expression,
    pub right: Expression,
    pub operator: Operator,
}

#[derive(PartialEq, Debug, Clone)]
pub enum VariableKind {
    /// `misal`
    Let,
    /// `konstan`
    Const,
}

#[derive(PartialEq, Debug, Clone)]
pub struct VariableDeclaration {
    pub kind: VariableKind,
    pub id: Identifier,
    pub value: Expression,
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionDeclaration {
    pub id: Identifier,
    pub params: Vec<Identifier>,
    pub body: BlockStatement,
    pub is_async: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Method {
    pub name: String,
    pub params: Vec<Identifier>,
    pub body: BlockStatement,
    pub is_async: bool,
}

/// `turunan Hewan(nama)`: the class being extended and the arguments its
/// constructor is called with.
#[derive(PartialEq, Debug, Clone)]
pub struct ParentClass {
    pub id: Identifier,
    pub arguments: Vec<Expression>,
}

/// `Kucing(nama) turunan Hewan(nama) { suara() { } }`
///
/// The statements in the body make up the constructor, in order. Methods are
/// collected separately.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct ClassDeclaration {
    pub id: Identifier,
    pub params: Vec<Identifier>,
    pub parent: Option<ParentClass>,
    pub body: Vec<Statement>,
    /// Source lines of `body`, like `BlockStatement::lines`.
    pub body_lines: Lines,
    pub methods: Vec<Method>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct CatchClause {
    pub param: Option<Identifier>,
    pub body: BlockStatement,
}

/// `coba { } tangkap galat { } akhirnya { }`
#[derive(PartialEq, Debug, Clone)]
pub struct TryStatement {
    pub block: BlockStatement,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStatement>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AlternateStatement {
    IfStatement(Box<IfStatement>),
    BlockStatement(BlockStatement),
}

#[derive(PartialEq, Debug, Clone)]
pub struct IfStatement {
    pub test: Expression,
    pub consequent: BlockStatement,
    pub alternate: Option<AlternateStatement>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct WhileStatement {
    pub test: Expression,
    pub body: BlockStatement,
}

/// `untuk i dari 1 sampai 10 langkah 2 { }`, both ends inclusive
#[derive(PartialEq, Debug, Clone)]
pub struct ForRangeStatement {
    pub var: Identifier,
    pub from: Expression,
    pub to: Expression,
    pub step: Option<Expression>,
    pub body: BlockStatement,
}

/// `untuk setiap x dalam daftar { }`
#[derive(PartialEq, Debug, Clone)]
pub struct ForEachStatement {
    pub var: Identifier,
    pub iterable: Expression,
    pub body: BlockStatement,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SwitchCase {
    pub tests: Vec<Expression>,
    pub body: BlockStatement,
}

/// `pilih x { kalau 1, 2 { } lain { } }`
#[derive(PartialEq, Debug, Clone)]
pub struct SwitchStatement {
    pub discriminant: Expression,
    pub cases: Vec<SwitchCase>,
    pub default: Option<BlockStatement>,
}

/// The source line (1-based) each statement of a block starts on, in order.
/// Used for error messages and never takes part in comparing syntax trees.
#[derive(Debug, Clone, Default)]
pub struct Lines(pub Vec<usize>);

impl PartialEq for Lines {
    fn eq(&self, _: &Lines) -> bool {
        true
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct BlockStatement {
    pub body: Option<Vec<Statement>>,
    pub lines: Lines,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Statement {
    Break,
    Continue,
    Return(Option<Expression>),
    Expression(Expression),
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ClassDeclaration),
    Try(TryStatement),
    Throw(Expression),
    BlockStatement(BlockStatement),
    Loop(BlockStatement),
    While(WhileStatement),
    ForRange(ForRangeStatement),
    ForEach(ForEachStatement),
    Switch(SwitchStatement),
    IfStatement(IfStatement),
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Program {
    pub body: Vec<Statement>,
    pub lines: Lines,
}
