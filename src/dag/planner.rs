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

    pub fn detect_project_type(spec: &str) -> &'static str {
        let lower = spec.to_lowercase();
        if lower.contains("website")
            || lower.contains(" web ")
            || lower.contains("frontend")
            || lower.contains("api")
            || lower.contains("rest")
            || lower.contains("http")
        {
            "web"
        } else if lower.contains("game")
            || lower.contains("pygame")
            || lower.contains("unity")
            || lower.contains("sprite")
        {
            "game"
        } else {
            "default"
        }
    }

    pub fn from_spec(spec_content: &str) -> BtcResult<TaskGraph> {
        let project_type = Self::detect_project_type(spec_content);
        let mut graph = Self::from_template(project_type)?;

        // Enrich each node with the full spec and task-specific instructions
        for node in graph.graph.node_weights_mut() {
            let task_instructions = match node.task_type {
                TaskType::Scaffold => {
                    "Create the project directory structure, initialize package manager (package.json, Cargo.toml, etc.), \
                     install dependencies, and create configuration files. Write all files to disk. \
                     Set up the build system so the project can compile/run."
                }
                TaskType::Code => {
                    "Implement the code for this component. Write all source files to disk. \
                     Follow the spec requirements exactly. Create working, functional code — not stubs or placeholders. \
                     Include proper error handling and follow best practices for the chosen tech stack."
                }
                TaskType::Asset => {
                    "Generate all required assets (images, icons, SVGs, CSS, etc.) and write them to disk. \
                     Create visually appropriate assets that match the project's design requirements."
                }
                TaskType::Test => {
                    "Write and run tests for the implemented code. Create test files covering \
                     the main functionality, edge cases, and error conditions. Run the tests and fix any failures."
                }
                TaskType::VisualQA => {
                    "Review all generated files for correctness. Check that the project builds/compiles successfully. \
                     Verify the code matches the spec requirements. Fix any issues found."
                }
                TaskType::Polish => {
                    "Final review and polish. Clean up code formatting, add missing comments where needed, \
                     ensure all files are consistent. Verify the project builds and runs correctly. \
                     Do a final check against the spec requirements."
                }
            };

            node.prompt = format!(
                "You are working on task: {}\n\n\
                 ## Task Instructions\n{}\n\n\
                 ## Full Project Specification\n{}\n\n\
                 IMPORTANT: You MUST write actual files to disk. Do not just describe what to do — \
                 create the files using the Write tool or by writing code. \
                 The project should be buildable and functional after this task completes.",
                node.name,
                task_instructions,
                spec_content
            );
        }

        Ok(graph)
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
