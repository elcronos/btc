use std::path::PathBuf;

use chrono::Utc;
use uuid::Uuid;

use crate::error::{BtcError, BtcResult};

use super::types::CronJob;

/// Scheduler for managing cron-based skill execution.
///
/// Placeholder -- actual tokio-cron-scheduler integration deferred.
pub struct CronScheduler {
    jobs: Vec<CronJob>,
    state_dir: PathBuf,
}

impl CronScheduler {
    pub fn new(state_dir: PathBuf) -> Self {
        Self {
            jobs: Vec::new(),
            state_dir,
        }
    }

    /// Add a new cron job and return its ID.
    pub fn add_job(&mut self, schedule: &str, skill: &str) -> BtcResult<String> {
        let id = Uuid::new_v4().to_string()[..8].to_string();
        let job = CronJob {
            id: id.clone(),
            schedule: schedule.to_string(),
            skill_name: skill.to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
            created_at: Utc::now(),
        };
        self.jobs.push(job);
        self.save_state()?;
        Ok(id)
    }

    /// Remove a cron job by ID.
    pub fn remove_job(&mut self, id: &str) -> BtcResult<()> {
        let before = self.jobs.len();
        self.jobs.retain(|j| j.id != id);
        if self.jobs.len() == before {
            return Err(BtcError::Cron(format!("Job not found: {id}")));
        }
        self.save_state()?;
        Ok(())
    }

    /// List all cron jobs.
    pub fn list_jobs(&self) -> &[CronJob] {
        &self.jobs
    }

    /// Persist jobs to a JSON file in the state directory.
    pub fn save_state(&self) -> BtcResult<()> {
        std::fs::create_dir_all(&self.state_dir)?;
        let path = self.state_dir.join("cron_jobs.json");
        let json = serde_json::to_string_pretty(&self.jobs)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load jobs from the JSON state file.
    pub fn load_state(&mut self) -> BtcResult<()> {
        let path = self.state_dir.join("cron_jobs.json");
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            self.jobs = serde_json::from_str(&contents)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_add_and_list_jobs() {
        let dir = TempDir::new().unwrap();
        let mut scheduler = CronScheduler::new(dir.path().to_path_buf());

        let id = scheduler.add_job("0 * * * *", "deploy").unwrap();
        assert!(!id.is_empty());
        assert_eq!(scheduler.list_jobs().len(), 1);
        assert_eq!(scheduler.list_jobs()[0].skill_name, "deploy");
        assert_eq!(scheduler.list_jobs()[0].schedule, "0 * * * *");
        assert!(scheduler.list_jobs()[0].enabled);
    }

    #[test]
    fn test_remove_job() {
        let dir = TempDir::new().unwrap();
        let mut scheduler = CronScheduler::new(dir.path().to_path_buf());

        let id = scheduler.add_job("0 * * * *", "deploy").unwrap();
        assert_eq!(scheduler.list_jobs().len(), 1);

        scheduler.remove_job(&id).unwrap();
        assert_eq!(scheduler.list_jobs().len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_job() {
        let dir = TempDir::new().unwrap();
        let mut scheduler = CronScheduler::new(dir.path().to_path_buf());

        let result = scheduler.remove_job("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_state() {
        let dir = TempDir::new().unwrap();
        let mut scheduler = CronScheduler::new(dir.path().to_path_buf());

        scheduler.add_job("0 * * * *", "deploy").unwrap();
        scheduler.add_job("30 2 * * *", "backup").unwrap();

        let mut scheduler2 = CronScheduler::new(dir.path().to_path_buf());
        scheduler2.load_state().unwrap();
        assert_eq!(scheduler2.list_jobs().len(), 2);
        assert_eq!(scheduler2.list_jobs()[0].skill_name, "deploy");
        assert_eq!(scheduler2.list_jobs()[1].skill_name, "backup");
    }
}
