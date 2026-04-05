use std::path::PathBuf;

use crate::error::BtcResult;

use super::browser::HeadlessBrowser;
use super::capture::ScreenshotCapture;
use super::checkpoint::CheckpointManager;
use super::consistency::ConsistencyChecker;
use super::scorer::VisualScorer;
use super::types::{Checkpoint, QaConfig, VisualScore};

/// Runs the core QA loop: execute -> capture -> score -> checkpoint -> compare.
pub struct QaLoopRunner {
    checkpoint_mgr: CheckpointManager,
    config: QaConfig,
    fallback_mode: bool,
}

impl QaLoopRunner {
    /// Create a new loop runner. Automatically detects whether a browser is
    /// available and enables fallback mode if not.
    pub fn new(repo_dir: PathBuf, config: QaConfig) -> Self {
        let fallback_mode = !HeadlessBrowser::is_available();
        if fallback_mode {
            tracing::warn!("Browser not available — running in fallback mode (no visual scoring)");
        }

        Self {
            checkpoint_mgr: CheckpointManager::new(repo_dir),
            config,
            fallback_mode,
        }
    }

    /// Run the QA loop, calling `execute_fn` each iteration to perform the
    /// actual task work.
    ///
    /// Returns the best checkpoint after the loop completes.
    pub fn run_loop(
        &mut self,
        mut execute_fn: impl FnMut() -> BtcResult<()>,
    ) -> BtcResult<Checkpoint> {
        for iteration in 0..self.config.max_retries {
            tracing::info!(iteration = iteration, "QA loop iteration");

            // 1. Execute the task
            execute_fn()?;

            // 2. Score
            let scores = if self.fallback_mode {
                self.placeholder_scores()
            } else {
                self.capture_and_score()?
            };

            // 3. Check consistency
            let mean = ConsistencyChecker::check(&scores)?;
            let consistent =
                ConsistencyChecker::is_consistent(&scores, self.config.cross_page_threshold);

            tracing::info!(
                mean = mean,
                consistent = consistent,
                "Scores evaluated"
            );

            // 4. Create checkpoint
            let checkpoint = self
                .checkpoint_mgr
                .create_checkpoint(iteration, scores)?;

            // 5. Check if threshold is met
            if checkpoint.aggregate_score >= self.config.page_score_threshold && consistent {
                tracing::info!(
                    score = checkpoint.aggregate_score,
                    "Threshold met — finishing QA loop"
                );
                return Ok(checkpoint);
            }

            // 6. If score dropped from best, note for potential rollback
            if let Some(best) = self.checkpoint_mgr.best_checkpoint() {
                if checkpoint.aggregate_score < best.aggregate_score {
                    tracing::warn!(
                        current = checkpoint.aggregate_score,
                        best = best.aggregate_score,
                        "Score regression detected"
                    );
                }
            }
        }

        // Return the best checkpoint after exhausting retries
        match self.checkpoint_mgr.best_checkpoint() {
            Some(best) => Ok(best.clone()),
            None => Err(crate::error::BtcError::VisualQa(
                "No checkpoints were created during QA loop".into(),
            )),
        }
    }

    fn capture_and_score(&self) -> BtcResult<Vec<VisualScore>> {
        let output_dir = self
            .checkpoint_mgr
            .checkpoints()
            .len()
            .to_string();
        let capture_dir = PathBuf::from(".btc").join("screenshots").join(output_dir);
        let capture = ScreenshotCapture::new(self.config.viewport, capture_dir)?;

        let paths = capture.capture_routes(&self.config.routes)?;
        let scorer = VisualScorer::new();
        let mut scores = Vec::with_capacity(paths.len());

        for path in &paths {
            let score = scorer.score(path, "visual-qa")?;
            scores.push(score);
        }

        Ok(scores)
    }

    fn placeholder_scores(&self) -> Vec<VisualScore> {
        use chrono::Utc;
        use crate::types::Score;

        self.config
            .routes
            .iter()
            .map(|route| VisualScore {
                page: route.clone(),
                score: Score::new(95.0),
                feedback: "Fallback mode — no visual scoring".into(),
                timestamp: Utc::now(),
            })
            .collect()
    }
}
