use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::claude::subprocess::{ClaudeProcess, SpawnOptions};
use crate::dag::graph::TaskGraph;
use crate::dag::node::TaskStatus;
use crate::error::BtcResult;
use crate::sandbox::NoopSandbox;
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
    debug: bool,
}

impl DagExecutor {
    pub fn new(graph: TaskGraph, project_dir: PathBuf) -> Self {
        Self {
            graph,
            project_dir,
            debug: false,
        }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Main execution loop. Dispatches each ready node to Claude.
    pub async fn execute(&mut self) -> BtcResult<ExecutionReport> {
        let start = Instant::now();
        let mut completed = Vec::new();
        let mut failed = Vec::new();

        loop {
            let ready = self.graph.ready_nodes();
            if ready.is_empty() {
                break;
            }

            for id in ready {
                let (name, prompt) = self
                    .graph
                    .get_node(&id)
                    .map(|n| (n.name.clone(), n.prompt.clone()))
                    .unwrap_or_else(|| (id.0.clone(), String::new()));

                println!("  → Starting task: {}", name);
                self.graph.mark_running(&id)?;

                let sandbox = NoopSandbox;
                let project_dir_str = self.project_dir.display().to_string();
                let system_prompt = format!(
                    "You are a software engineer executing a build task. \
                     The project directory is: {project_dir_str}\n\
                     ALL files MUST be written inside this directory using ABSOLUTE paths starting with {project_dir_str}/.\n\
                     Do not ask questions — just implement. Create real, working files.\n\
                     Use the Write tool with absolute paths like {project_dir_str}/index.html."
                );
                let opts = SpawnOptions {
                    model: None,
                    system_prompt: Some(&system_prompt),
                    extra_flags: vec![],
                };

                let result = async {
                    let mut proc = ClaudeProcess::spawn(
                        &prompt,
                        &sandbox,
                        &self.project_dir,
                        opts,
                    )
                    .await?;

                    loop {
                        match proc.read_event().await? {
                            Some(event) => {
                                if self.debug {
                                    println!("  [debug] {:?}", event);
                                }
                            }
                            None => break,
                        }
                    }
                    Ok::<(), crate::error::BtcError>(())
                }
                .await;

                match result {
                    Ok(_) => {
                        println!("  ✓ Completed: {}", name);
                        self.graph.mark_complete(&id)?;
                        completed.push(id);
                    }
                    Err(e) => {
                        println!("  ✗ Failed: {} — {}", name, e);
                        self.graph.mark_failed(&id)?;
                        failed.push(id);
                    }
                }
            }

            if self.graph.is_complete() {
                break;
            }

            // Check for deadlock (no progress possible)
            let has_progress = self
                .graph
                .graph
                .node_weights()
                .any(|n| n.status == TaskStatus::Ready || n.status == TaskStatus::Running);
            if !has_progress {
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
