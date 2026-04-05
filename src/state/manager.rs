use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tempfile::NamedTempFile;

use crate::error::{BtcError, BtcResult};

pub struct StateManager {
    state_dir: PathBuf,
}

impl StateManager {
    pub fn new(state_dir: PathBuf) -> BtcResult<Self> {
        let manager = Self { state_dir };
        manager.ensure_dir()?;
        Ok(manager)
    }

    pub fn read<T: DeserializeOwned>(&self, key: &str) -> BtcResult<T> {
        let path = self.state_dir.join(format!("{}.json", key));
        let contents = fs::read_to_string(&path).map_err(|e| {
            BtcError::State(format!("failed to read state key '{}': {}", key, e))
        })?;
        serde_json::from_str(&contents).map_err(|e| {
            BtcError::State(format!("failed to parse state key '{}': {}", key, e))
        })
    }

    pub fn write<T: Serialize>(&self, key: &str, value: &T) -> BtcResult<()> {
        let target = self.state_dir.join(format!("{}.json", key));
        let data = serde_json::to_string_pretty(value).map_err(|e| {
            BtcError::State(format!("failed to serialize state key '{}': {}", key, e))
        })?;

        // Atomic write: temp file in same dir -> fsync -> rename
        let mut tmp = NamedTempFile::new_in(&self.state_dir).map_err(|e| {
            BtcError::State(format!("failed to create temp file: {}", e))
        })?;
        tmp.write_all(data.as_bytes())?;
        tmp.as_file().sync_all()?;
        tmp.persist(&target).map_err(|e| {
            BtcError::State(format!("failed to persist state key '{}': {}", key, e))
        })?;

        Ok(())
    }

    pub fn exists(&self, key: &str) -> bool {
        self.state_dir.join(format!("{}.json", key)).exists()
    }

    pub fn delete(&self, key: &str) -> BtcResult<()> {
        let path = self.state_dir.join(format!("{}.json", key));
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                BtcError::State(format!("failed to delete state key '{}': {}", key, e))
            })?;
        }
        Ok(())
    }

    fn ensure_dir(&self) -> BtcResult<()> {
        let subdirs = ["checkpoints", "cron", "agents"];
        fs::create_dir_all(&self.state_dir)?;
        for sub in &subdirs {
            fs::create_dir_all(self.state_dir.join(sub))?;
        }
        Ok(())
    }
}
