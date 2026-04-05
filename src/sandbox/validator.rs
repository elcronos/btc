use std::path::{Path, PathBuf};

use crate::error::{BtcError, BtcResult};

pub struct PathValidator {
    project_dir: PathBuf,
}

impl PathValidator {
    pub fn new(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }

    /// Validate that a path (after canonicalization) lives inside the project directory.
    /// Returns `Ok(true)` if valid, or `Err(SandboxViolation)` if outside.
    pub fn validate(&self, path: &Path) -> BtcResult<bool> {
        let canonical = self.canonicalize_and_check(path)?;
        // If canonicalize_and_check succeeded the path is within project_dir.
        let _ = canonical;
        Ok(true)
    }

    /// Canonicalize the path (resolving symlinks) and verify it starts with
    /// the project directory. Returns the canonical path on success.
    pub fn canonicalize_and_check(&self, path: &Path) -> BtcResult<PathBuf> {
        let canonical = path.canonicalize().map_err(|e| {
            BtcError::Sandbox(format!(
                "failed to canonicalize path '{}': {e}",
                path.display()
            ))
        })?;

        let project_canonical = self.project_dir.canonicalize().map_err(|e| {
            BtcError::Sandbox(format!(
                "failed to canonicalize project dir '{}': {e}",
                self.project_dir.display()
            ))
        })?;

        if canonical.starts_with(&project_canonical) {
            Ok(canonical)
        } else {
            Err(BtcError::SandboxViolation { path: path.to_path_buf() })
        }
    }
}
