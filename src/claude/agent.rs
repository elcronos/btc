use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::error::BtcResult;
use crate::sandbox::Sandbox;
use crate::types::{AgentId, AgentSnapshot, AgentStatus};

use super::stream::StreamEvent;
use super::subprocess::{ClaudeProcess, SpawnOptions};

const MAX_EVENTS: usize = 1000;

/// A running Claude Code agent with its subprocess, event history, and metadata.
pub struct Agent {
    pub id: AgentId,
    pub status: AgentStatus,
    pub task: String,
    process: ClaudeProcess,
    pub cost: f64,
    events: VecDeque<StreamEvent>,
    pub started_at: DateTime<Utc>,
}

impl Agent {
    /// Spawn a new agent that runs the given prompt via Claude Code.
    pub async fn spawn(
        task: String,
        prompt: &str,
        sandbox: &dyn Sandbox,
        project_dir: &Path,
        opts: SpawnOptions<'_>,
    ) -> BtcResult<Self> {
        let process = ClaudeProcess::spawn(prompt, sandbox, project_dir, opts).await?;

        Ok(Self {
            id: AgentId::new(),
            status: AgentStatus::Running,
            task,
            process,
            cost: 0.0,
            events: VecDeque::with_capacity(MAX_EVENTS),
            started_at: Utc::now(),
        })
    }

    /// Read all currently available events from the subprocess and
    /// append them to the ring buffer. Updates status based on events.
    pub async fn poll_events(&mut self) -> BtcResult<Vec<StreamEvent>> {
        let mut new_events = Vec::new();

        loop {
            // Use a short timeout so we don't block forever waiting
            // for the next event — return whatever is available now.
            let maybe_event = tokio::time::timeout(
                Duration::from_millis(10),
                self.process.read_event(),
            )
            .await;

            match maybe_event {
                Ok(Ok(Some(event))) => {
                    // Track cost from result events.
                    if let Some(c) = event.cost() {
                        self.cost += c;
                    }

                    // Update status based on event type.
                    match &event {
                        StreamEvent::Error { .. } => {
                            self.status = AgentStatus::Error;
                        }
                        StreamEvent::ResultMessage { .. } => {
                            self.status = AgentStatus::Complete;
                        }
                        _ => {}
                    }

                    // Append to ring buffer, dropping oldest if full.
                    if self.events.len() >= MAX_EVENTS {
                        self.events.pop_front();
                    }
                    self.events.push_back(event.clone());
                    new_events.push(event);
                }
                Ok(Ok(None)) => {
                    // EOF — process has finished.
                    if self.status == AgentStatus::Running {
                        self.status = AgentStatus::Complete;
                    }
                    break;
                }
                Ok(Err(e)) => {
                    tracing::error!(agent = %self.id, "Read error: {e}");
                    self.status = AgentStatus::Error;
                    break;
                }
                Err(_timeout) => {
                    // No more events available right now.
                    break;
                }
            }
        }

        Ok(new_events)
    }

    /// Kill the agent's subprocess.
    pub async fn kill(&mut self) -> BtcResult<()> {
        self.process.kill(Duration::from_secs(5)).await?;
        self.status = AgentStatus::Error;
        Ok(())
    }

    /// Duration since the agent was spawned.
    pub fn elapsed(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }

    /// Build a snapshot for TUI display.
    pub fn snapshot(&self) -> AgentSnapshot {
        let last_output = self
            .events
            .iter()
            .rev()
            .find_map(|e| e.text().map(|t| t.to_string()));

        AgentSnapshot {
            id: self.id.clone(),
            status: self.status,
            task: self.task.clone(),
            cost: self.cost,
            duration_secs: self.elapsed().num_milliseconds() as f64 / 1000.0,
            event_count: self.events.len(),
            last_output,
        }
    }

    /// Check if the underlying process is still running.
    pub fn is_running(&mut self) -> bool {
        self.process.is_running()
    }
}
