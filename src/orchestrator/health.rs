use std::time::Duration;

use crate::claude::Agent;
use crate::types::{AgentId, AgentStatus};

pub struct HealthChecker {
    pub heartbeat_timeout: Duration,
    pub check_interval: Duration,
}

impl HealthChecker {
    pub fn new(heartbeat_timeout: Duration, check_interval: Duration) -> Self {
        Self {
            heartbeat_timeout,
            check_interval,
        }
    }

    /// Returns IDs of agents that appear stuck — running longer than
    /// `heartbeat_timeout` without producing new events.
    pub fn check_agents(&self, agents: &[Agent]) -> Vec<AgentId> {
        let timeout_secs = self.heartbeat_timeout.as_secs() as i64;

        agents
            .iter()
            .filter(|agent| {
                agent.status == AgentStatus::Running
                    && agent.elapsed().num_seconds() > timeout_secs
            })
            .map(|agent| agent.id.clone())
            .collect()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(Duration::from_secs(60), Duration::from_secs(10))
    }
}
