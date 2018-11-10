mod types;
mod visitor;

use super::super::WindowParams;
use super::Visitor;

use errors::*;

pub fn new(params: WindowParams) -> Result<Box<Visitor>> {
    let visitor = self::visitor::GlutinVisitor::new(params)?;
    Ok(Box::new(visitor))
}
