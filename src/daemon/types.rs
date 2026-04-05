use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonCommand {
    Status,
    Pause,
    Resume,
    Approve,
    Cancel,
    Run(String),
    CronAdd { schedule: String, skill: String },
    CronRemove(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    Ok(String),
    Error(String),
    Status(DaemonStatus),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub active_agents: usize,
    pub total_cost: f64,
    pub uptime_secs: f64,
    pub cron_jobs: usize,
}
