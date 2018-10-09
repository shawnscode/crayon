use std::sync::Arc;

use crayon::errors::*;
use crayon::res::registry::Register;

use super::bytecode::*;

#[derive(Clone)]
pub struct BytecodeLoader {}

impl BytecodeLoader {
    pub fn new() -> Self {
        BytecodeLoader {}
    }
}

impl Register for BytecodeLoader {
    type Handle = BytecodeHandle;
    type Intermediate = Bytecode;
    type Value = Arc<Bytecode>;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        info!(
            "[BytecodeLoader] loads {:?}, {} bytes.",
            handle,
            bytes.len()
        );

        Ok(Bytecode(String::from_utf8_lossy(&bytes).into()))
    }

    fn attach(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Value> {
        info!("[BytecodeLoader] attach {:?}.", handle);
        Ok(Arc::new(item))
    }

    fn detach(&self, handle: Self::Handle, _: Self::Value) {
        info!("[BytecodeLoader] detach {:?}.", handle);
    }
}
