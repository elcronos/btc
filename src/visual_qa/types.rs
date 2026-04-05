use crate::types::{CheckpointId, Score};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualScore {
    pub page: String,
    pub score: Score,
    pub feedback: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub git_commit: String,
    pub scores: Vec<VisualScore>,
    pub aggregate_score: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaConfig {
    pub page_score_threshold: f64,
    pub cross_page_threshold: f64,
    pub max_retries: u32,
    pub viewport: (u32, u32),
    pub routes: Vec<String>,
}

impl Default for QaConfig {
    fn default() -> Self {
        Self {
            page_score_threshold: 90.0,
            cross_page_threshold: 85.0,
            max_retries: 3,
            viewport: (1280, 720),
            routes: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointComparison {
    pub current: Checkpoint,
    pub best: Checkpoint,
    pub score_delta: f64,
    pub should_rollback: bool,
}
