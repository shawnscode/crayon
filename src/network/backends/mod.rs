use crate::errors::Result;
pub trait Visitor {
    fn create_connection(&mut self, params: String)
        -> Result<()>;
    fn poll_events(&mut self,v:&mut Vec<String>);
    fn send(&mut self,v:String);
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tokio;

#[cfg(not(target_arch = "wasm32"))]
pub fn new() -> Result<Box<Visitor>> {
    let visitor = self::tokio::visitor::TokioVisitor::new()?;
    Ok(Box::new(visitor))
}

#[cfg(target_arch = "wasm32")]
pub mod websys;

#[cfg(target_arch = "wasm32")]
pub fn new() -> Result<Box<Visitor>> {
    let visitor = websys::visitor::WebVisitor::new()?;
    Ok(Box::new(visitor))
}