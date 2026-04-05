use std::path::Path;

use crate::error::BtcResult;

pub trait Sandbox: Send + Sync {
    /// Validate that a path is within the allowed project directory.
    fn validate_path(&self, path: &Path, project_dir: &Path) -> BtcResult<bool>;

    /// Wrap a command to run under sandbox restrictions.
    fn wrap_command(
        &self,
        cmd: &mut tokio::process::Command,
        project_dir: &Path,
    ) -> BtcResult<()>;

    /// Get a human-readable name for this sandbox implementation.
    fn name(&self) -> &'static str;
}
