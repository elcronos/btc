use serde::{Deserialize, Serialize};

use crate::dag::graph::TaskGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusState {
    Proposing,
    Reviewing,
    Critiquing,
    Agreed,
    Deadlocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub iteration: u32,
    pub state: ConsensusState,
    pub planner_output: String,
    pub architect_review: String,
    pub critic_verdict: String,
}

#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub dag: TaskGraph,
    pub rounds: Vec<ConsensusRound>,
    pub consensus_reached: bool,
}
