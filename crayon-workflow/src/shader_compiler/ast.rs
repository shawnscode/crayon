use super::lex::*;

pub type Program = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    PriorVariable(PriorVariable),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PriorVariable {
    pub qualifier: Qualifier,
    pub tt: Type,
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub ret: Type,
    pub params: Vec<(Type, String)>,
    pub codes: Vec<FunctionStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionStatement {}


// #[derive(Debug, Clone ,PartialEq)]
// pub enum Expression {
//     /// Identifier expression.
//     Ident(Ident),
//     /// Literal expressions.
//     Literial(Literial),
//     /// A unary expression, gathering a single expression and a unary operator.
//     Unary(UnaryOperator, Box<Expression>),
//     /// A binary expression, gathering two expressions and a binary operator.
//     Binary(BinaryOperator, Box<Expression>, Box<Expression>),
//     /// A ternary conditional expression, gathering three expressions.
//     If(Box<Expression>, Vec<Statement>, Option<Vec<Statement>>),
//     /// A function definition
//     FunDef(Vec<Ident>, Vec<Statement>),
//     /// A functional call. It has a function identifier and a list of expressions (arguments).
//     FunCall(Ident, Vec<Expression>),
//     ///
//     Index(Box<Expression>, Box<Expression>),
// }

// pub type Ident = String;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum TypeIdent {
//     Void,
//     Int,
//     Float,
//     Vec2,
//     Vec3,
//     Vec4,
//     Mat2,
//     Mat3,
//     Mat4,
//     Sampler2D,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum VariableQualifier {
//     Attribute,
//     Uniform,
//     Varying,
// }

// /// All the unary opartors.
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum UnaryOperator {
//     /// `++ expr`
//     Inc,
//     /// `-- expr`
//     Dec,
//     /// `+ expr`
//     Add,
//     /// `- expr`
//     Minus,
//     /// `! expr`
//     Not,
//     /// `~ expr`
//     Complement,
// }

// /// All the binary operators.
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum BinaryOperator {
//     /// ` expr1 || expr2 `
//     Or,
//     /// ` expr1 && expr2 `
//     And,
//     /// ` expr1 | expr2 `
//     BitOr,
//     /// ` expr1 ^ expr2 `
//     BitXor,
//     /// ` expr1 & expr2 `
//     BitAnd,
//     /// ` expr1 == expr2 `
//     Equal,
//     /// ` expr1 != expr2 `
//     NonEqual,
//     /// ` expr1 < expr2`
//     LT,
//     /// ` expr1 > expr2`
//     GT,
//     /// ` expr1 <= expr2`
//     LTE,
//     /// ` expr1 >= expr2`
//     GTE,
//     /// ` expr1 << expr2`
//     LShift,
//     /// ` expr1 >> expr2`
//     RShift,
//     /// ` expr1 + expr2`
//     Add,
//     /// ` expr1 - expr2`
//     Sub,
//     /// ` expr1 * expr2`
//     Mul,
//     /// ` expr1 / expr2`
//     Div,
//     /// ` expr1 % expr2`
//     Mod,
// }