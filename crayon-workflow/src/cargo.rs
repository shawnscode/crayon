use std::process::{Command, Stdio};
use errors::*;

/// Executes Cargo with the provided arguments. Returns a failure string if
/// Cargo couldn't be run.
pub fn call(args: &[&str]) -> Result<()> {
    let mut command = Command::new("cargo");

    for arg in args {
        command.arg(arg);
    }

    let exec_result = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output();

    if let Ok(output) = exec_result {
        if output.status.success() {
            Ok(())
        } else {
            bail!("Failed to run cargo.");
        }
    } else {
        bail!("Failed to run cargo.");
    }
}