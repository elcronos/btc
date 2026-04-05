use std::time::Duration;

use crate::claude::Agent;
use crate::error::BtcResult;

pub struct GracefulShutdown {
    pub timeout: Duration,
}

impl GracefulShutdown {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Kill all agents, waiting up to `self.timeout` for the entire operation.
    pub async fn shutdown_all(&self, agents: &mut Vec<Agent>) -> BtcResult<()> {
        let result = tokio::time::timeout(self.timeout, async {
            for agent in agents.iter_mut() {
                match agent.kill().await {
                    Ok(()) => {
                        tracing::info!(agent = %agent.id, "Agent shut down");
                    }
                    Err(e) => {
                        tracing::warn!(agent = %agent.id, "Failed to shut down agent: {e}");
                    }
                }
            }
        })
        .await;

        if result.is_err() {
            tracing::warn!("Graceful shutdown timed out after {:?}", self.timeout);
        }

        Ok(())
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}
