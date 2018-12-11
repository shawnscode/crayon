mod types;
mod visitor;

use super::Visitor;

use crate::errors::*;
use crate::window::WindowParams;

pub fn new(params: WindowParams) -> Result<Box<Visitor>> {
    let visitor = visitor::WebVisitor::new(params)?;
    Ok(Box::new(visitor))
}
