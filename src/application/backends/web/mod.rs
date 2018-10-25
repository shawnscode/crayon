pub mod sys;
mod visitor;

use super::Visitor;

use application::settings::WindowParams;
use errors::*;

pub fn new(params: WindowParams) -> Result<Box<Visitor>> {
    let visitor = visitor::WebVisitor::new(params)?;
    Ok(Box::new(visitor))
}
