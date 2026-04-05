use std::path::Path;

use crate::dag::graph::TaskGraph;
use crate::error::BtcResult;

pub fn save_graph(graph: &TaskGraph, path: &Path) -> BtcResult<()> {
    let json = serde_json::to_string_pretty(graph)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_graph(path: &Path) -> BtcResult<TaskGraph> {
    let data = std::fs::read_to_string(path)?;
    let graph: TaskGraph = serde_json::from_str(&data)?;
    Ok(graph)
}
