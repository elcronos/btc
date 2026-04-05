use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::dag::node::{TaskNode, TaskStatus};
use crate::error::{BtcError, BtcResult};
use crate::types::TaskId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    pub graph: DiGraph<TaskNode, ()>,
    #[serde(
        serialize_with = "serialize_node_indices",
        deserialize_with = "deserialize_node_indices"
    )]
    pub node_indices: HashMap<TaskId, NodeIndex>,
}

fn serialize_node_indices<S: Serializer>(
    map: &HashMap<TaskId, NodeIndex>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let raw: HashMap<&TaskId, usize> = map.iter().map(|(k, v)| (k, v.index())).collect();
    raw.serialize(s)
}

fn deserialize_node_indices<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<HashMap<TaskId, NodeIndex>, D::Error> {
    let raw: HashMap<TaskId, usize> = HashMap::deserialize(d)?;
    Ok(raw.into_iter().map(|(k, v)| (k, NodeIndex::new(v))).collect())
}

impl TaskGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, node: TaskNode) -> TaskId {
        let id = node.id.clone();
        let idx = self.graph.add_node(node);
        self.node_indices.insert(id.clone(), idx);
        id
    }

    /// Adds a dependency edge: `from` must complete before `to` can run.
    pub fn add_dependency(&mut self, from: &TaskId, to: &TaskId) -> BtcResult<()> {
        let from_idx = self
            .node_indices
            .get(from)
            .ok_or_else(|| BtcError::Dag(format!("Node not found: {}", from)))?;
        let to_idx = self
            .node_indices
            .get(to)
            .ok_or_else(|| BtcError::Dag(format!("Node not found: {}", to)))?;
        self.graph.add_edge(*from_idx, *to_idx, ());
        Ok(())
    }

    /// Returns task IDs where all predecessors are Complete and status is Ready.
    pub fn ready_nodes(&self) -> Vec<TaskId> {
        let mut ready = Vec::new();
        for (id, idx) in &self.node_indices {
            let node = &self.graph[*idx];
            if node.status != TaskStatus::Ready {
                continue;
            }
            let all_preds_complete = self
                .graph
                .neighbors_directed(*idx, Direction::Incoming)
                .all(|pred_idx| self.graph[pred_idx].status == TaskStatus::Complete);
            if all_preds_complete {
                ready.push(id.clone());
            }
        }
        ready
    }

    /// Marks a node as Complete and promotes any newly-ready dependents to Ready.
    pub fn mark_complete(&mut self, id: &TaskId) -> BtcResult<()> {
        let idx = self
            .node_indices
            .get(id)
            .copied()
            .ok_or_else(|| BtcError::Dag(format!("Node not found: {}", id)))?;
        self.graph[idx].status = TaskStatus::Complete;
        self.promote_dependents(idx);
        Ok(())
    }

    pub fn mark_failed(&mut self, id: &TaskId) -> BtcResult<()> {
        let idx = self
            .node_indices
            .get(id)
            .copied()
            .ok_or_else(|| BtcError::Dag(format!("Node not found: {}", id)))?;
        self.graph[idx].status = TaskStatus::Failed;
        Ok(())
    }

    pub fn mark_running(&mut self, id: &TaskId) -> BtcResult<()> {
        let idx = self
            .node_indices
            .get(id)
            .copied()
            .ok_or_else(|| BtcError::Dag(format!("Node not found: {}", id)))?;
        self.graph[idx].status = TaskStatus::Running;
        Ok(())
    }

    pub fn reset_to_ready(&mut self, id: &TaskId) -> BtcResult<()> {
        let idx = self
            .node_indices
            .get(id)
            .copied()
            .ok_or_else(|| BtcError::Dag(format!("Node not found: {}", id)))?;
        self.graph[idx].status = TaskStatus::Ready;
        Ok(())
    }

    pub fn get_node(&self, id: &TaskId) -> Option<&TaskNode> {
        self.node_indices.get(id).map(|idx| &self.graph[*idx])
    }

    pub fn get_node_mut(&mut self, id: &TaskId) -> Option<&mut TaskNode> {
        self.node_indices
            .get(id)
            .copied()
            .map(move |idx| &mut self.graph[idx])
    }

    pub fn is_complete(&self) -> bool {
        self.graph.node_weights().all(|n| {
            n.status == TaskStatus::Complete || n.status == TaskStatus::Skipped
        })
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn completed_count(&self) -> usize {
        self.graph
            .node_weights()
            .filter(|n| n.status == TaskStatus::Complete)
            .count()
    }

    /// After a node completes, check its outgoing neighbors and promote them to Ready
    /// if all their predecessors are now Complete.
    fn promote_dependents(&mut self, completed_idx: NodeIndex) {
        let dependents: Vec<NodeIndex> = self
            .graph
            .neighbors_directed(completed_idx, Direction::Outgoing)
            .collect();

        for dep_idx in dependents {
            if self.graph[dep_idx].status != TaskStatus::Pending {
                continue;
            }
            let all_preds_complete = self
                .graph
                .neighbors_directed(dep_idx, Direction::Incoming)
                .all(|pred_idx| self.graph[pred_idx].status == TaskStatus::Complete);
            if all_preds_complete {
                self.graph[dep_idx].status = TaskStatus::Ready;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::criteria::{EntryCriteria, ExitCriteria};
    use crate::dag::node::{TaskNode, TaskStatus, TaskType};

    fn make_node(name: &str, status: TaskStatus) -> TaskNode {
        TaskNode {
            id: TaskId::from_str(name),
            name: name.to_string(),
            task_type: TaskType::Code,
            prompt: String::new(),
            entry_criteria: vec![EntryCriteria::AllPredecessorsComplete],
            exit_criteria: vec![ExitCriteria::BuildSucceeds],
            max_retries: 3,
            status,
            retries_used: 0,
        }
    }

    #[test]
    fn test_add_nodes_and_dependencies() {
        let mut g = TaskGraph::new();
        let a = g.add_task(make_node("a", TaskStatus::Ready));
        let b = g.add_task(make_node("b", TaskStatus::Pending));
        let c = g.add_task(make_node("c", TaskStatus::Pending));

        g.add_dependency(&a, &b).unwrap();
        g.add_dependency(&a, &c).unwrap();

        assert_eq!(g.node_count(), 3);
        // Only "a" is Ready with no predecessors
        let ready = g.ready_nodes();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].0, "a");
    }

    #[test]
    fn test_mark_complete_propagates_readiness() {
        let mut g = TaskGraph::new();
        let a = g.add_task(make_node("a", TaskStatus::Ready));
        let b = g.add_task(make_node("b", TaskStatus::Pending));
        let c = g.add_task(make_node("c", TaskStatus::Pending));

        g.add_dependency(&a, &b).unwrap();
        g.add_dependency(&a, &c).unwrap();

        g.mark_running(&a).unwrap();
        assert!(g.ready_nodes().is_empty());

        g.mark_complete(&a).unwrap();
        let ready = g.ready_nodes();
        assert_eq!(ready.len(), 2);
        let mut names: Vec<&str> = ready.iter().map(|id| id.0.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["b", "c"]);
    }
}
