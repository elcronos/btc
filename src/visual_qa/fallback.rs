use std::path::Path;

use crate::error::BtcResult;

/// Fallback quality gates used when Chrome is not available.
/// Runs build and test commands as a substitute for visual scoring.
pub struct QaFallback;

impl QaFallback {
    /// Run build-based quality gates as a substitute for visual QA.
    /// Returns `true` if both `cargo build` and `cargo test` succeed.
    pub fn run_build_gates(project_dir: &Path) -> BtcResult<bool> {
        tracing::warn!(
            "Visual QA disabled — Chrome not available. Running build gates as fallback."
        );

        // Run cargo build
        let build = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(project_dir)
            .output()?;

        if !build.status.success() {
            tracing::error!(
                stderr = %String::from_utf8_lossy(&build.stderr),
                "cargo build failed"
            );
            return Ok(false);
        }

        tracing::info!("cargo build passed");

        // Run cargo test
        let test = std::process::Command::new("cargo")
            .arg("test")
            .current_dir(project_dir)
            .output()?;

        if !test.status.success() {
            tracing::error!(
                stderr = %String::from_utf8_lossy(&test.stderr),
                "cargo test failed"
            );
            return Ok(false);
        }

        tracing::info!("cargo test passed — build gates OK");
        Ok(true)
    }
}
