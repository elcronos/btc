use std::collections::HashMap;
use std::time::Duration;

use crate::types::AgentId;

pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_base_ms: u64,
    agent_retries: HashMap<AgentId, u32>,
}

impl RetryPolicy {
    pub fn new(max_retries: u32, backoff_base_ms: u64) -> Self {
        Self {
            max_retries,
            backoff_base_ms,
            agent_retries: HashMap::new(),
        }
    }

    /// Check whether the agent has retries remaining.
    pub fn should_retry(&self, agent_id: &AgentId) -> bool {
        let count = self.agent_retries.get(agent_id).copied().unwrap_or(0);
        count < self.max_retries
    }

    /// Record a failure for an agent, incrementing its retry count.
    pub fn record_failure(&mut self, agent_id: &AgentId) {
        let count = self.agent_retries.entry(agent_id.clone()).or_insert(0);
        *count += 1;
    }

    /// Reset the retry count for an agent.
    pub fn reset(&mut self, agent_id: &AgentId) {
        self.agent_retries.remove(agent_id);
    }

    /// Compute exponential backoff duration: base_ms * 2^retries.
    pub fn backoff_duration(&self, agent_id: &AgentId) -> Duration {
        let retries = self.agent_retries.get(agent_id).copied().unwrap_or(0);
        let ms = self.backoff_base_ms * 2u64.pow(retries);
        Duration::from_millis(ms)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3, 1000)
    }
}
