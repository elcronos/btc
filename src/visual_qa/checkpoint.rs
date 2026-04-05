use std::path::PathBuf;

use chrono::Utc;

use crate::error::{BtcError, BtcResult};
use crate::state::GitignoreEnforcer;
use crate::types::CheckpointId;

use super::types::{Checkpoint, VisualScore};

/// Manages git-based checkpoints with score tracking.
pub struct CheckpointManager {
    repo_dir: PathBuf,
    checkpoints: Vec<Checkpoint>,
    best: Option<usize>,
}

impl CheckpointManager {
    pub fn new(repo_dir: PathBuf) -> Self {
        Self {
            repo_dir,
            checkpoints: Vec::new(),
            best: None,
        }
    }

    /// Create a checkpoint by committing current state to git.
    /// Verifies `.btc/` is gitignored before committing.
    pub fn create_checkpoint(
        &mut self,
        iteration: u32,
        scores: Vec<VisualScore>,
    ) -> BtcResult<Checkpoint> {
        // Safety: ensure .btc/ is gitignored before any git operations
        GitignoreEnforcer::ensure_btc_ignored(&self.repo_dir)?;

        let commit_msg = format!("btc-checkpoint-{}", iteration);

        // Stage all changes
        let add_output = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&self.repo_dir)
            .output()?;

        if !add_output.status.success() {
            return Err(BtcError::Checkpoint(format!(
                "git add failed: {}",
                String::from_utf8_lossy(&add_output.stderr)
            )));
        }

        // Commit
        let commit_output = std::process::Command::new("git")
            .args(["commit", "-m", &commit_msg, "--allow-empty"])
            .current_dir(&self.repo_dir)
            .output()?;

        if !commit_output.status.success() {
            return Err(BtcError::Checkpoint(format!(
                "git commit failed: {}",
                String::from_utf8_lossy(&commit_output.stderr)
            )));
        }

        // Get the commit hash
        let rev_output = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_dir)
            .output()?;

        let git_commit = String::from_utf8_lossy(&rev_output.stdout).trim().to_string();

        let aggregate_score = if scores.is_empty() {
            0.0
        } else {
            let sum: f64 = scores.iter().map(|s| s.score.value()).sum();
            sum / scores.len() as f64
        };

        let checkpoint = Checkpoint {
            id: CheckpointId::new(iteration),
            git_commit,
            scores,
            aggregate_score,
            created_at: Utc::now(),
        };

        self.checkpoints.push(checkpoint.clone());
        let idx = self.checkpoints.len() - 1;
        self.update_best_index(idx);

        tracing::info!(
            checkpoint = %checkpoint.id,
            score = checkpoint.aggregate_score,
            "Checkpoint created"
        );

        Ok(checkpoint)
    }

    /// Roll back to the best-scoring checkpoint via `git reset --hard`.
    /// Verifies `.btc/` is gitignored first.
    pub fn rollback_to_best(&self) -> BtcResult<()> {
        GitignoreEnforcer::ensure_btc_ignored(&self.repo_dir)?;

        let best = self
            .best_checkpoint()
            .ok_or_else(|| BtcError::Checkpoint("No best checkpoint to rollback to".into()))?;

        tracing::info!(
            checkpoint = %best.id,
            commit = %best.git_commit,
            "Rolling back to best checkpoint"
        );

        let output = std::process::Command::new("git")
            .args(["reset", "--hard", &best.git_commit])
            .current_dir(&self.repo_dir)
            .output()?;

        if !output.status.success() {
            return Err(BtcError::Checkpoint(format!(
                "git reset failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Return a reference to the best checkpoint, if any.
    pub fn best_checkpoint(&self) -> Option<&Checkpoint> {
        self.best.map(|idx| &self.checkpoints[idx])
    }

    /// Update the best checkpoint if the given checkpoint has a higher aggregate score.
    pub fn update_best(&mut self, checkpoint: &Checkpoint) {
        // Find the checkpoint in our list by id
        if let Some(idx) = self
            .checkpoints
            .iter()
            .position(|c| c.id == checkpoint.id)
        {
            self.update_best_index(idx);
        }
    }

    /// Return all checkpoints.
    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    fn update_best_index(&mut self, idx: usize) {
        match self.best {
            None => {
                self.best = Some(idx);
            }
            Some(current_best) => {
                if self.checkpoints[idx].aggregate_score
                    > self.checkpoints[current_best].aggregate_score
                {
                    self.best = Some(idx);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Score;
    use chrono::Utc;

    fn make_score(page: &str, value: f64) -> VisualScore {
        VisualScore {
            page: page.to_string(),
            score: Score::new(value),
            feedback: String::new(),
            timestamp: Utc::now(),
        }
    }

    fn make_checkpoint(iteration: u32, aggregate: f64) -> Checkpoint {
        Checkpoint {
            id: CheckpointId::new(iteration),
            git_commit: format!("abc{}", iteration),
            scores: vec![],
            aggregate_score: aggregate,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_best_checkpoint_tracking() {
        let dir = tempfile::tempdir().unwrap();
        let mut mgr = CheckpointManager::new(dir.path().to_path_buf());

        // Manually push checkpoints (bypass git for unit test)
        let cp1 = make_checkpoint(1, 80.0);
        mgr.checkpoints.push(cp1.clone());
        mgr.update_best_index(0);

        assert_eq!(mgr.best_checkpoint().unwrap().aggregate_score, 80.0);

        let cp2 = make_checkpoint(2, 92.0);
        mgr.checkpoints.push(cp2.clone());
        mgr.update_best_index(1);

        assert_eq!(mgr.best_checkpoint().unwrap().aggregate_score, 92.0);

        // Lower score should not replace best
        let cp3 = make_checkpoint(3, 85.0);
        mgr.checkpoints.push(cp3.clone());
        mgr.update_best_index(2);

        assert_eq!(mgr.best_checkpoint().unwrap().aggregate_score, 92.0);
        assert_eq!(mgr.best_checkpoint().unwrap().id, CheckpointId::new(2));
    }

    #[test]
    fn test_no_best_initially() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = CheckpointManager::new(dir.path().to_path_buf());
        assert!(mgr.best_checkpoint().is_none());
    }

    #[test]
    fn test_update_best_via_public_api() {
        let dir = tempfile::tempdir().unwrap();
        let mut mgr = CheckpointManager::new(dir.path().to_path_buf());

        let cp1 = make_checkpoint(1, 70.0);
        mgr.checkpoints.push(cp1.clone());
        mgr.update_best(&cp1);

        assert_eq!(mgr.best_checkpoint().unwrap().aggregate_score, 70.0);

        let cp2 = make_checkpoint(2, 95.0);
        mgr.checkpoints.push(cp2.clone());
        mgr.update_best(&cp2);

        assert_eq!(mgr.best_checkpoint().unwrap().aggregate_score, 95.0);

        // Lower score via update_best should not change
        let cp3 = make_checkpoint(3, 60.0);
        mgr.checkpoints.push(cp3.clone());
        mgr.update_best(&cp3);

        assert_eq!(mgr.best_checkpoint().unwrap().aggregate_score, 95.0);
    }

    #[test]
    fn test_checkpoints_list() {
        let dir = tempfile::tempdir().unwrap();
        let mut mgr = CheckpointManager::new(dir.path().to_path_buf());

        mgr.checkpoints.push(make_checkpoint(1, 80.0));
        mgr.checkpoints.push(make_checkpoint(2, 90.0));

        assert_eq!(mgr.checkpoints().len(), 2);
    }
}
