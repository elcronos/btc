use std::path::Path;

use chrono::Utc;

use crate::error::BtcResult;
use crate::types::Score;

use super::types::VisualScore;

/// Scores screenshots for visual quality.
///
/// In production this would send the screenshot to Claude Code through
/// `AgentSupervisor` for vision-based scoring. For now it returns a
/// placeholder score.
pub struct VisualScorer;

impl VisualScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score a screenshot.
    /// Currently returns a mock passing score.
    pub fn score(&self, screenshot_path: &Path, context: &str) -> BtcResult<VisualScore> {
        tracing::debug!(
            path = %screenshot_path.display(),
            context = context,
            "Scoring screenshot (placeholder)"
        );

        let page = screenshot_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(VisualScore {
            page,
            score: Score::new(95.0),
            feedback: "Placeholder score — vision scoring not yet implemented".into(),
            timestamp: Utc::now(),
        })
    }
}

impl Default for VisualScorer {
    fn default() -> Self {
        Self::new()
    }
}
