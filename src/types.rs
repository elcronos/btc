use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string()[..8].to_string())
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Spawning,
    Running,
    Idle,
    Error,
    Complete,
}

impl AgentStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Spawning => "[NEW]",
            Self::Running => "[RUN]",
            Self::Idle => "[IDLE]",
            Self::Error => "[ERR]",
            Self::Complete => "[OK]",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Spawning => "magenta",
            Self::Running => "green",
            Self::Idle => "yellow",
            Self::Error => "red",
            Self::Complete => "blue",
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string()[..8].to_string())
    }

    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Score(pub f64);

impl Score {
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 100.0))
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn passes_threshold(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointId(pub String);

impl CheckpointId {
    pub fn new(iteration: u32) -> Self {
        Self(format!("checkpoint-{}", iteration))
    }
}

impl fmt::Display for CheckpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub id: AgentId,
    pub status: AgentStatus,
    pub task: String,
    pub cost: f64,
    pub duration_secs: f64,
    pub event_count: usize,
    pub last_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_agents: usize,
    pub running_agents: usize,
    pub total_cost: f64,
    pub elapsed_secs: f64,
    pub checkpoints_created: u32,
    pub best_score: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            total_agents: 0,
            running_agents: 0,
            total_cost: 0.0,
            elapsed_secs: 0.0,
            checkpoints_created: 0,
            best_score: None,
            timestamp: Utc::now(),
        }
    }
}
