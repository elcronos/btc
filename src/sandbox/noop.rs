use std::path::Path;

use crate::error::BtcResult;
use super::traits::Sandbox;

pub struct NoopSandbox;

impl Sandbox for NoopSandbox {
    fn validate_path(&self, _path: &Path, _project_dir: &Path) -> BtcResult<bool> {
        tracing::warn!("sandbox is not active — path validation skipped");
        Ok(true)
    }

    fn wrap_command(
        &self,
        _cmd: &mut tokio::process::Command,
        _project_dir: &Path,
    ) -> BtcResult<()> {
        tracing::warn!("sandbox is not active — command not sandboxed");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "noop"
    }
}
