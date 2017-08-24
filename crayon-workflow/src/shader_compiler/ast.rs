use super::lex::*;

pub type Program = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    PriorVariable(PriorVariable),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorVariable {
    pub qualifier: Qualifier,
    pub tt: Type,
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub ident: String,
    pub ret: Type,
    pub params: Vec<(Type, String)>,
    pub codes: Vec<FunctionStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionStatement {
    VariableBind(VariableBind),
    If(If),
    Expression(Expression),
    Return(Return),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableBind {
    pub tt: Option<Type>,
    pub ident: String,
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub cond: Box<Expression>,
    pub consequence: Vec<FunctionStatement>,
    pub alternative: Option<Vec<FunctionStatement>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// Identifier expression.
    Ident(String),
    /// Literal expressions.
    Literial(Literial),
    /// A unary expression, gathering a single expression and a unary operator.
    Unary(UnaryOperator, Box<Expression>),
    /// A binary expression, gathering two expressions and a binary operator.
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
    /// A functional call. It has a function identifier and a list of expressions (arguments).
    Call(Box<Expression>, Vec<Expression>),
    /// Index
    Index(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Lowest,
    Eq,
    Comparations,
    Sum,
    Product,
    Call,
    Index,
}

/// All the unary opartors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// `++ expr`
    Inc,
    /// `-- expr`
    Dec,
    /// `+ expr`
    Add,
    /// `- expr`
    Minus,
    /// `! expr`
    Not,
    /// `~ expr`
    Complement,
}

/// All the binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// ` expr1 || expr2 `
    Or,
    /// ` expr1 && expr2 `
    And,
    /// ` expr1 | expr2 `
    BitOr,
    /// ` expr1 ^ expr2 `
    BitXor,
    /// ` expr1 & expr2 `
    BitAnd,
    /// ` expr1 == expr2 `
    Eq,
    /// ` expr1 != expr2 `
    Neq,
    /// ` expr1 < expr2`
    LT,
    /// ` expr1 > expr2`
    GT,
    /// ` expr1 <= expr2`
    LTE,
    /// ` expr1 >= expr2`
    GTE,
    /// ` expr1 << expr2`
    LShift,
    /// ` expr1 >> expr2`
    RShift,
    /// ` expr1 + expr2`
    Add,
    /// ` expr1 - expr2`
    Minus,
    /// ` expr1 * expr2`
    Mul,
    /// ` expr1 / expr2`
    Div,
    /// ` expr1 % expr2`
    Mod,
}