use std::sync::Arc;

use crayon::errors::Result;
use crayon::res::utils::prelude::ResourceLoader;

impl_handle!(BytesHandle);

#[derive(Clone)]
pub struct BytesLoader {}

impl BytesLoader {
    pub(crate) fn new() -> Self {
        BytesLoader {}
    }
}

impl ResourceLoader for BytesLoader {
    type Handle = BytesHandle;
    type Intermediate = Vec<u8>;
    type Resource = Vec<u8>;

    fn load(&self, _handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        let data = bytes.to_vec();
        info!(
            "[BytesLoader] data: {:?}).",
            data.clone()
        );

        Ok(data)
    }

    fn create(&self, _: Self::Handle, item: Self::Intermediate) -> Result<Self::Resource> {
        Ok(item.clone())
    }

    fn delete(&self, _: Self::Handle, _: Self::Resource) {}
}
