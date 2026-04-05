use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::AgentId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub hook_type: HookType,
    pub agent_id: Option<AgentId>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    SessionStart,
    Stop,
    SubagentStart,
    SubagentStop,
}
