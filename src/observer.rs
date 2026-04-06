use std::path::{Path, PathBuf};
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Path where hooks write events
pub fn event_log_path(project_dir: &Path) -> PathBuf {
    project_dir.join(".btc").join("agent-events.jsonl")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub hook: String,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_type: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub cwd: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TrackedAgent {
    pub id: String,
    pub agent_type: String,
    pub session_id: String,
    pub status: AgentTrackStatus,
    pub tools_used: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentTrackStatus {
    Running,
    Completed,
}

/// Read the event log and reconstruct agent topology
pub fn read_agent_topology(project_dir: &Path) -> Vec<TrackedAgent> {
    let log_path = event_log_path(project_dir);
    if !log_path.exists() {
        return Vec::new();
    }

    let file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut agents: std::collections::HashMap<String, TrackedAgent> = std::collections::HashMap::new();
    let mut main_session: Option<String> = None;

    for line in reader.lines().flatten() {
        let event: AgentEvent = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Track main session
        if main_session.is_none() {
            if let Some(ref sid) = event.session_id {
                main_session = Some(sid.clone());
            }
        }

        match event.hook.as_str() {
            "SubagentStart" => {
                if let (Some(aid), Some(atype)) = (event.agent_id.clone(), event.agent_type.clone()) {
                    agents.insert(aid.clone(), TrackedAgent {
                        id: aid,
                        agent_type: atype,
                        session_id: event.session_id.unwrap_or_default(),
                        status: AgentTrackStatus::Running,
                        tools_used: Vec::new(),
                        started_at: Some(Utc::now()),
                        stopped_at: None,
                    });
                }
            }
            "SubagentStop" => {
                if let Some(aid) = event.agent_id {
                    if let Some(agent) = agents.get_mut(&aid) {
                        agent.status = AgentTrackStatus::Completed;
                        agent.stopped_at = Some(Utc::now());
                    }
                }
            }
            "PreToolUse" | "PostToolUse" => {
                if let Some(ref tool) = event.tool_name {
                    // Associate tool use with the latest running agent or main session
                    // For simplicity, track tool names on the main agent
                    let key = event.session_id.clone().unwrap_or_else(|| "main".to_string());
                    agents.entry(key.clone()).or_insert_with(|| TrackedAgent {
                        id: key,
                        agent_type: "main".to_string(),
                        session_id: event.session_id.unwrap_or_default(),
                        status: AgentTrackStatus::Running,
                        tools_used: Vec::new(),
                        started_at: Some(Utc::now()),
                        stopped_at: None,
                    }).tools_used.push(tool.clone());
                }
            }
            _ => {}
        }
    }

    agents.into_values().collect()
}

/// Get total token/cost estimate from events (count tool uses as proxy)
pub fn estimate_usage(project_dir: &Path) -> (usize, usize) {
    let log_path = event_log_path(project_dir);
    if !log_path.exists() {
        return (0, 0);
    }

    let content = match std::fs::read_to_string(&log_path) {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let total_events = content.lines().count();
    let tool_uses = content.lines()
        .filter(|l| l.contains("\"PreToolUse\""))
        .count();

    (total_events, tool_uses)
}
