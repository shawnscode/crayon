mod visitor;

use super::Visitor;

use errors::*;
use window::WindowParams;

pub fn new(params: WindowParams) -> Result<Box<Visitor>> {
    let visitor = visitor::WebVisitor::new(params)?;
    Ok(Box::new(visitor))
}
