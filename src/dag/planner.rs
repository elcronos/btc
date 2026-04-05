use crate::dag::criteria::{EntryCriteria, ExitCriteria};
use crate::dag::graph::TaskGraph;
use crate::dag::node::{TaskNode, TaskStatus, TaskType};
use crate::error::BtcResult;
use crate::types::TaskId;

pub struct DagPlanner;

impl DagPlanner {
    fn make_node(name: &str, task_type: TaskType, prompt: &str, status: TaskStatus) -> TaskNode {
        TaskNode {
            id: TaskId::from_str(name),
            name: name.to_string(),
            task_type,
            prompt: prompt.to_string(),
            entry_criteria: vec![EntryCriteria::AllPredecessorsComplete],
            exit_criteria: vec![ExitCriteria::BuildSucceeds],
            max_retries: 3,
            status,
            retries_used: 0,
        }
    }

    pub fn from_template(project_type: &str) -> BtcResult<TaskGraph> {
        let mut g = TaskGraph::new();

        match project_type {
            "game" => {
                let scaffold = g.add_task(Self::make_node(
                    "scaffold",
                    TaskType::Scaffold,
                    "Create project scaffold",
                    TaskStatus::Ready,
                ));
                let core_logic = g.add_task(Self::make_node(
                    "core_logic",
                    TaskType::Code,
                    "Implement core game logic",
                    TaskStatus::Pending,
                ));
                let assets = g.add_task(Self::make_node(
                    "assets",
                    TaskType::Asset,
                    "Generate game assets",
                    TaskStatus::Pending,
                ));
                let ui = g.add_task(Self::make_node(
                    "ui_integration",
                    TaskType::Code,
                    "Integrate UI components",
                    TaskStatus::Pending,
                ));
                let vqa = g.add_task(Self::make_node(
                    "visual_qa",
                    TaskType::VisualQA,
                    "Visual QA pass",
                    TaskStatus::Pending,
                ));
                let polish = g.add_task(Self::make_node(
                    "polish",
                    TaskType::Polish,
                    "Final polish",
                    TaskStatus::Pending,
                ));

                g.add_dependency(&scaffold, &core_logic)?;
                g.add_dependency(&scaffold, &assets)?;
                g.add_dependency(&core_logic, &ui)?;
                g.add_dependency(&assets, &ui)?;
                g.add_dependency(&ui, &vqa)?;
                g.add_dependency(&vqa, &polish)?;
            }
            "web" => {
                let scaffold = g.add_task(Self::make_node(
                    "scaffold",
                    TaskType::Scaffold,
                    "Create project scaffold",
                    TaskStatus::Ready,
                ));
                let backend = g.add_task(Self::make_node(
                    "backend",
                    TaskType::Code,
                    "Implement backend",
                    TaskStatus::Pending,
                ));
                let frontend = g.add_task(Self::make_node(
                    "frontend",
                    TaskType::Code,
                    "Implement frontend",
                    TaskStatus::Pending,
                ));
                let integration = g.add_task(Self::make_node(
                    "integration",
                    TaskType::Test,
                    "Integration testing",
                    TaskStatus::Pending,
                ));
                let vqa = g.add_task(Self::make_node(
                    "visual_qa",
                    TaskType::VisualQA,
                    "Visual QA pass",
                    TaskStatus::Pending,
                ));
                let polish = g.add_task(Self::make_node(
                    "polish",
                    TaskType::Polish,
                    "Final polish",
                    TaskStatus::Pending,
                ));

                g.add_dependency(&scaffold, &backend)?;
                g.add_dependency(&scaffold, &frontend)?;
                g.add_dependency(&backend, &integration)?;
                g.add_dependency(&frontend, &integration)?;
                g.add_dependency(&integration, &vqa)?;
                g.add_dependency(&vqa, &polish)?;
            }
            _ => {
                let scaffold = g.add_task(Self::make_node(
                    "scaffold",
                    TaskType::Scaffold,
                    "Create project scaffold",
                    TaskStatus::Ready,
                ));
                let implementation = g.add_task(Self::make_node(
                    "implementation",
                    TaskType::Code,
                    "Implement features",
                    TaskStatus::Pending,
                ));
                let test = g.add_task(Self::make_node(
                    "test",
                    TaskType::Test,
                    "Run tests",
                    TaskStatus::Pending,
                ));
                let polish = g.add_task(Self::make_node(
                    "polish",
                    TaskType::Polish,
                    "Final polish",
                    TaskStatus::Pending,
                ));

                g.add_dependency(&scaffold, &implementation)?;
                g.add_dependency(&implementation, &test)?;
                g.add_dependency(&test, &polish)?;
            }
        }

        Ok(g)
    }

    /// Placeholder: falls back to default template.
    pub fn from_consensus(_spec: &str) -> BtcResult<TaskGraph> {
        Self::from_template("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_template_game() {
        let g = DagPlanner::from_template("game").unwrap();
        assert_eq!(g.node_count(), 6);

        // scaffold should be the only ready node
        let ready = g.ready_nodes();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].0, "scaffold");

        // Verify node names exist
        assert!(g.get_node(&TaskId::from_str("scaffold")).is_some());
        assert!(g.get_node(&TaskId::from_str("core_logic")).is_some());
        assert!(g.get_node(&TaskId::from_str("assets")).is_some());
        assert!(g.get_node(&TaskId::from_str("ui_integration")).is_some());
        assert!(g.get_node(&TaskId::from_str("visual_qa")).is_some());
        assert!(g.get_node(&TaskId::from_str("polish")).is_some());
    }
}
