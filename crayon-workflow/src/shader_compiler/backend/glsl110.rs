use super::super::lex::*;
use super::super::syntax::*;

use std::fmt::{Write, Result};

struct Writer<'a, W: Write + 'static> {
    writer: &'a mut W,
    indent: u32,
}

impl<'a, W: Write + 'static> Writer<'a, W> {
    pub fn write(&mut self, v: &str) -> Result {
        self.writer.write_str(v)
    }

    pub fn newline(&mut self) -> Result {
        self.writer.write_str("\n")?;
        for _ in 0..self.indent {
            self.writer.write_str("\t")?;
        }
        Ok(())
    }

    pub fn push(&mut self) {
        self.indent += 1;
    }

    pub fn pop(&mut self) {
        self.indent -= 1;
    }
}

pub fn write<W>(w: &mut W, program: &Vec<Statement>) -> Result
    where W: Write + 'static
{
    let mut wrapper = Writer {
        writer: w,
        indent: 0,
    };

    wrapper.write("#version 110")?;
    wrapper.newline()?;

    for v in program {
        match v {
            &Statement::PriorVariable(ref v) => write_prior_variable(&mut wrapper, v)?,
            &Statement::Function(ref v) => write_function(&mut wrapper, v)?,
            _ => {}
        }
    }

    Ok(())
}

fn write_prior_variable<'a, W>(mut w: &mut Writer<'a, W>, pv: &statement::PriorVariable) -> Result
    where W: Write + 'static
{
    write_qualifier(&mut w, &pv.qualifier)?;
    w.write(" ")?;
    write_type(w, &pv.tt)?;
    w.write(" ")?;
    w.write(&pv.ident)?;
    w.write(";")?;
    w.newline()?;

    Ok(())
}

fn write_function<'a, W>(mut w: &mut Writer<'a, W>, f: &statement::Function) -> Result
    where W: Write + 'static
{
    write_type(&mut w, &f.ret)?;
    w.write(" ")?;
    w.write(&f.ident)?;
    w.write("(")?;

    for (i, &(ref tt, ref param)) in f.params.iter().enumerate() {
        write_type(&mut w, tt)?;
        w.write(" ")?;
        w.write(param.as_ref())?;

        if i < (f.params.len() - 1) {
            w.write(", ")?;
        }
    }

    w.write(") {")?;
    write_block(&mut w, &f.codes)?;
    w.write("}")?;
    w.newline()?;
    Ok(())
}

fn write_block<'a, W>(mut w: &mut Writer<'a, W>,
                      stmts: &Vec<statement::FunctionStatement>)
                      -> Result
    where W: Write + 'static
{
    w.push();
    w.newline()?;
    for stmt in stmts {
        write_function_statement(&mut w, &stmt)?;
    }
    w.pop();
    w.newline()?;

    Ok(())
}

fn write_function_statement<'a, W>(mut w: &mut Writer<'a, W>,
                                   stmt: &statement::FunctionStatement)
                                   -> Result
    where W: Write + 'static
{
    match *stmt {
        statement::FunctionStatement::VariableBind(ref v) => write_variable_bind(&mut w, v)?,
        statement::FunctionStatement::Expression(ref v) => {
            write_expr(&mut w, v)?;
            w.write(";")?;
            w.newline()?;
        }
        statement::FunctionStatement::Return(ref v) => write_return(&mut w, v)?,
        statement::FunctionStatement::If(ref v) => write_if(&mut w, v)?,
    }

    Ok(())
}

fn write_variable_bind<'a, W>(mut w: &mut Writer<'a, W>, bind: &statement::VariableBind) -> Result
    where W: Write + 'static
{
    if let Some(tt) = bind.tt {
        write_type(w, &tt)?;
        w.write(" ")?;
    }

    w.write(&bind.ident)?;
    w.write(" = ")?;
    write_expr(&mut w, &bind.expr)?;
    w.write(";")?;
    w.newline()?;

    Ok(())
}

fn write_return<'a, W>(mut w: &mut Writer<'a, W>, ret: &statement::Return) -> Result
    where W: Write + 'static
{
    w.write("return ")?;
    write_expr(&mut w, &ret.expr)?;
    w.write(";")?;
    w.newline()?;
    Ok(())
}

fn write_if<'a, W>(mut w: &mut Writer<'a, W>, v: &statement::If) -> Result
    where W: Write + 'static
{
    w.write("if( ")?;
    write_expr(&mut w, &v.cond)?;
    w.write(" ) {")?;

    write_block(&mut w, &v.consequence)?;

    if let Some(ref alternative) = v.alternative {
        w.write("} else {")?;
        write_block(&mut w, alternative)?;
    }

    w.write("}")?;
    w.newline()?;

    Ok(())
}

fn write_expr<'a, W>(mut w: &mut Writer<'a, W>, expr: &Expression) -> Result
    where W: Write + 'static
{
    match *expr {
        Expression::Ident(ref v) => w.write(v)?,
        Expression::Literial(ref v) => {
            match *v {
                Literial::Int(vv) => w.write(&vv.to_string())?,
                Literial::Float(vv) => w.write(&vv.to_string())?,
            }
        }
        Expression::Unary(op, ref rhs) => {
            w.write("(")?;
            write_unary_op(&mut w, op)?;
            write_expr(&mut w, &rhs)?;
            w.write(")")?;
        }
        Expression::Binary(op, ref lhs, ref rhs) => {
            w.write("(")?;
            write_expr(&mut w, &lhs)?;
            write_binary_op(&mut w, op)?;
            write_expr(&mut w, &rhs)?;
            w.write(")")?;
        }
        Expression::Construct(ref lhs, ref params) => {
            write_type(&mut w, lhs)?;
            write_params(&mut w, &params)?;
        }
        Expression::Call(ref lhs, ref params) => {
            write_expr(&mut w, lhs)?;
            write_params(&mut w, &params)?;
        }
        Expression::Index(ref lhs, ref rhs) => {
            write_expr(&mut w, lhs)?;
            w.write("[")?;
            write_expr(&mut w, rhs)?;
            w.write("]")?;
        }
        Expression::Dot(ref lhs, ref rhs) => {
            write_expr(&mut w, lhs)?;
            w.write(".")?;
            w.write(&rhs)?;
        }
    }

    Ok(())
}

fn write_params<'a, W>(mut w: &mut Writer<'a, W>, params: &Vec<Expression>) -> Result
    where W: Write + 'static
{
    w.write("(")?;

    for (i, ref param) in params.iter().enumerate() {
        write_expr(w, param)?;
        if i < (params.len() - 1) {
            w.write(", ")?;
        }
    }

    w.write(")")?;

    Ok(())
}

fn write_qualifier<'a, W>(mut w: &mut Writer<'a, W>, qualifier: &Qualifier) -> Result
    where W: Write + 'static
{
    match *qualifier {
        Qualifier::Attribute => w.write("attribute")?,
        Qualifier::Uniform => w.write("uniform")?,
        Qualifier::Varying => w.write("varying")?,
    }

    Ok(())
}

fn write_type<'a, W>(mut w: &mut Writer<'a, W>, tt: &Type) -> Result
    where W: Write + 'static
{
    match *tt {
        Type::Void => w.write("void")?,
        Type::Int => w.write("int")?,
        Type::Float => w.write("float")?,
        Type::Vec2 => w.write("vec2")?,
        Type::Vec3 => w.write("vec3")?,
        Type::Vec4 => w.write("vec4")?,
        Type::Mat2 => w.write("mat2")?,
        Type::Mat3 => w.write("mat3")?,
        Type::Mat4 => w.write("mat4")?,
        Type::Sampler2D => w.write("sampler2D")?,
    }

    Ok(())
}

fn write_unary_op<'a, W>(mut w: &mut Writer<'a, W>, op: expr::UnaryOperator) -> Result
    where W: Write + 'static
{
    match op {
        expr::UnaryOperator::Inc => w.write("++")?,
        expr::UnaryOperator::Dec => w.write("++")?,
        expr::UnaryOperator::Add => w.write("+")?,
        expr::UnaryOperator::Minus => w.write("-")?,
        expr::UnaryOperator::Not => w.write("!")?,
        expr::UnaryOperator::Complement => w.write("~")?,
    }

    Ok(())
}

fn write_binary_op<'a, W>(mut w: &mut Writer<'a, W>, op: expr::BinaryOperator) -> Result
    where W: Write + 'static
{
    use self::expr::BinaryOperator as ebo;
    match op {
        ebo::Or => w.write("||")?,
        ebo::And => w.write("&&")?,
        ebo::BitOr => w.write("|")?,
        ebo::BitXor => w.write("^")?,
        ebo::BitAnd => w.write("&")?,
        ebo::Eq => w.write("==")?,
        ebo::Neq => w.write("!=")?,
        ebo::LT => w.write("<")?,
        ebo::GT => w.write(">")?,
        ebo::LTE => w.write("<=")?,
        ebo::GTE => w.write(">=")?,
        ebo::LShift => w.write("<<")?,
        ebo::RShift => w.write(">>")?,
        ebo::Add => w.write("+")?,
        ebo::Minus => w.write("-")?,
        ebo::Mul => w.write("*")?,
        ebo::Div => w.write("/")?,
        ebo::Mod => w.write("%")?,
    }

    Ok(())
}