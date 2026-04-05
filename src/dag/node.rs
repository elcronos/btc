use crate::dag::criteria::{EntryCriteria, ExitCriteria};
use crate::types::TaskId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: TaskId,
    pub name: String,
    pub task_type: TaskType,
    pub prompt: String,
    pub entry_criteria: Vec<EntryCriteria>,
    pub exit_criteria: Vec<ExitCriteria>,
    pub max_retries: u32,
    pub status: TaskStatus,
    pub retries_used: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Scaffold,
    Code,
    Asset,
    Test,
    VisualQA,
    Polish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Ready,
    Running,
    Complete,
    Failed,
    Skipped,
}
