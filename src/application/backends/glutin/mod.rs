mod types;
mod visitor;

use super::Visitor;

use application::settings::WindowParams;
use errors::*;

pub fn new(params: WindowParams) -> Result<Box<Visitor>> {
    let visitor = self::visitor::GlutinVisitor::new(params)?;
    Ok(Box::new(visitor))
}

pub mod sys {
    use application::time::Instant;

    pub fn instant() -> Instant {
        let duration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        let ms = duration.subsec_millis() as u64 + duration.as_secs() * 1000;
        Instant::from_millis(ms)
    }
}
