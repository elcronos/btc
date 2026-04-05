use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryCriteria {
    AllPredecessorsComplete,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExitCriteria {
    TestsPassing,
    BuildSucceeds,
    VisualScoreAbove(f64),
    Custom(String),
}
