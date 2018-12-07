mod types;
mod visitor;

use super::super::WindowParams;
use super::Visitor;

use crate::errors::*;

pub fn new(params: WindowParams) -> Result<Box<Visitor>> {
    let visitor = self::visitor::GlutinVisitor::from(params)?;
    Ok(Box::new(visitor))
}
