use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::config::NetworkPolicy;
use crate::error::BtcResult;

use super::traits::Sandbox;
use super::validator::PathValidator;

pub struct MacOsSandbox {
    profile_path: PathBuf,
    validator: PathValidator,
    #[allow(dead_code)]
    network_policy: NetworkPolicy,
}

impl MacOsSandbox {
    pub fn new(
        profile_path: PathBuf,
        project_dir: PathBuf,
        network_policy: NetworkPolicy,
    ) -> Self {
        Self {
            profile_path,
            validator: PathValidator::new(project_dir),
            network_policy,
        }
    }
}

impl Sandbox for MacOsSandbox {
    fn validate_path(&self, path: &Path, _project_dir: &Path) -> BtcResult<bool> {
        self.validator.validate(path)
    }

    fn wrap_command(
        &self,
        cmd: &mut tokio::process::Command,
        _project_dir: &Path,
    ) -> BtcResult<()> {
        // Read the original program and args from the command, then rebuild
        // as: sandbox-exec -f <profile> <original_program> <original_args...>
        let original_program = cmd.as_std().get_program().to_os_string();
        let original_args: Vec<std::ffi::OsString> = cmd
            .as_std()
            .get_args()
            .map(OsStr::to_os_string)
            .collect();

        // Replace the command in-place: reset to sandbox-exec
        *cmd = tokio::process::Command::new("sandbox-exec");
        cmd.arg("-f")
            .arg(&self.profile_path)
            .arg(&original_program)
            .args(&original_args);

        tracing::debug!(
            profile = %self.profile_path.display(),
            "Wrapped command with macOS sandbox"
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "macos-sbpl"
    }
}
