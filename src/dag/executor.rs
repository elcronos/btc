use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::dag::graph::TaskGraph;
use crate::dag::node::TaskStatus;
use crate::error::BtcResult;
use crate::types::TaskId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub completed_nodes: Vec<TaskId>,
    pub failed_nodes: Vec<TaskId>,
    pub elapsed_secs: f64,
    pub total_cost: f64,
}

pub struct DagExecutor {
    pub graph: TaskGraph,
    pub project_dir: PathBuf,
}

impl DagExecutor {
    pub fn new(graph: TaskGraph, project_dir: PathBuf) -> Self {
        Self { graph, project_dir }
    }

    /// Main execution loop. Placeholder: marks each ready node as complete immediately.
    pub fn execute(&mut self) -> BtcResult<ExecutionReport> {
        let start = Instant::now();
        let mut completed = Vec::new();
        let mut failed = Vec::new();

        loop {
            let ready = self.graph.ready_nodes();
            if ready.is_empty() {
                break;
            }

            for id in ready {
                self.graph.mark_running(&id)?;
                // Placeholder dispatch: just mark complete
                self.graph.mark_complete(&id)?;
                completed.push(id);
            }

            if self.graph.is_complete() {
                break;
            }

            // Check if all remaining nodes are Failed (deadlock)
            let has_progress = self
                .graph
                .graph
                .node_weights()
                .any(|n| n.status == TaskStatus::Ready || n.status == TaskStatus::Running);
            if !has_progress {
                // Collect failed nodes
                for (id, idx) in &self.graph.node_indices {
                    if self.graph.graph[*idx].status == TaskStatus::Failed {
                        failed.push(id.clone());
                    }
                }
                break;
            }
        }

        Ok(ExecutionReport {
            completed_nodes: completed,
            failed_nodes: failed,
            elapsed_secs: start.elapsed().as_secs_f64(),
            total_cost: 0.0,
        })
    }
}
