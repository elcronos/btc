use std::path::{Path, PathBuf};

use crate::error::BtcResult;

/// Manages a temporary staging workspace for asset generation.
pub struct StagingWorkspace {
    root: PathBuf,
}

impl StagingWorkspace {
    /// Create a new staging workspace under `base_dir/{batch_id}/`.
    pub fn new(batch_id: &str, base_dir: &Path) -> BtcResult<Self> {
        let root = base_dir.join(batch_id);
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Return the path for an asset within this staging workspace.
    pub fn asset_path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    /// Copy a staged asset to the final project destination.
    pub fn promote_to_project(asset: &Path, destination: &Path) -> BtcResult<()> {
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(asset, destination)?;
        Ok(())
    }

    /// Remove the staging directory.
    pub fn cleanup(self) -> BtcResult<()> {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }

    /// Check whether the staging directory exists on disk.
    pub fn exists(&self) -> bool {
        self.root.exists()
    }

    /// Return the root path of this staging workspace.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn staging_lifecycle() {
        let tmp = TempDir::new().unwrap();
        let ws = StagingWorkspace::new("batch-001", tmp.path()).unwrap();
        assert!(ws.exists());

        // Write a test file into staging
        let asset = ws.asset_path("logo.svg");
        std::fs::write(&asset, "<svg/>").unwrap();
        assert!(asset.exists());

        // Promote to project destination
        let dest = tmp.path().join("project/assets/logo.svg");
        StagingWorkspace::promote_to_project(&asset, &dest).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "<svg/>");

        // Cleanup
        ws.cleanup().unwrap();
        assert!(!tmp.path().join("batch-001").exists());
    }
}
