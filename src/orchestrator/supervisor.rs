use std::sync::Arc;

use crate::claude::subprocess::SpawnOptions;
use crate::claude::Agent;
use crate::config::BtcConfig;
use crate::error::BtcResult;
use crate::sandbox::Sandbox;
use crate::state::StateManager;
use crate::types::{AgentId, AgentSnapshot, AgentStatus};

use super::health::HealthChecker;
use super::retry::RetryPolicy;
use super::shutdown::GracefulShutdown;

pub struct AgentSupervisor {
    agents: Vec<Agent>,
    sandbox: Arc<dyn Sandbox>,
    state: Arc<StateManager>,
    pub retry_policy: RetryPolicy,
    pub health_checker: HealthChecker,
    config: BtcConfig,
}

impl AgentSupervisor {
    pub fn new(sandbox: Arc<dyn Sandbox>, state: Arc<StateManager>, config: BtcConfig) -> Self {
        Self {
            agents: Vec::new(),
            sandbox,
            state,
            retry_policy: RetryPolicy::default(),
            health_checker: HealthChecker::default(),
            config,
        }
    }

    /// Spawn a new agent with the given task prompt and optional model override.
    pub async fn spawn_agent(
        &mut self,
        task: &str,
        model: Option<&str>,
    ) -> BtcResult<AgentId> {
        let opts = SpawnOptions {
            model,
            ..Default::default()
        };

        let agent = Agent::spawn(
            task.to_string(),
            task,
            self.sandbox.as_ref(),
            &self.config.project_dir,
            opts,
        )
        .await?;

        let id = agent.id.clone();
        tracing::info!(agent = %id, task = task, "Agent spawned");
        self.agents.push(agent);
        Ok(id)
    }

    /// Look up an agent by ID.
    pub fn get_agent(&self, id: &AgentId) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == *id)
    }

    /// Look up an agent by ID (mutable).
    pub fn get_agent_mut(&mut self, id: &AgentId) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == *id)
    }

    /// Kill a specific agent by ID.
    pub async fn kill_agent(&mut self, id: &AgentId) -> BtcResult<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == *id) {
            agent.kill().await?;
            tracing::info!(agent = %id, "Agent killed");
        }
        Ok(())
    }

    /// Return references to all agents that are not yet Complete.
    pub fn active_agents(&self) -> Vec<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.status != AgentStatus::Complete)
            .collect()
    }

    /// Build snapshots of every agent for TUI display.
    pub fn snapshots(&self) -> Vec<AgentSnapshot> {
        self.agents.iter().map(|a| a.snapshot()).collect()
    }

    /// Gracefully shut down all agents.
    pub async fn shutdown(&mut self) -> BtcResult<()> {
        let shutdown = GracefulShutdown::default();
        shutdown.shutdown_all(&mut self.agents).await
    }
}
