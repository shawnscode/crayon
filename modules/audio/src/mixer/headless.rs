use std::sync::{Arc, RwLock};
use std::thread::Builder;

use super::Command;
use crayon::errors::Result;

pub fn run(rx: Arc<RwLock<Vec<Command>>>) -> Result<()> {
    info!("Create headless audio mixer.",);

    Builder::new()
        .name("Audio".into())
        .spawn(move || {
            //
            loop {
                {
                    let mut rx = rx.write().unwrap();
                    rx.clear();
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }).expect("Failed to create thread for `AudioSystem`.");

    Ok(())
}
