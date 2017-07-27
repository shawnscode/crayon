extern crate crayon-cli;

use std::path::Path;
use std::fs;

struct Workspace {
    path: Path,
}

// impl Workspace {
//     fn new(&mut self) {}
// }

impl Drop for Workspace {
    fn drop(&mut self) {
        fs::remove_dir(&self.path);
    }
}

#[test]
fn refresh() {}