use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSpec {
    pub name: String,
    pub description: String,
    pub format: AssetFormat,
    pub dimensions: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetFormat {
    Svg,
    Png,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResult {
    pub path: PathBuf,
    pub score: f64,
    pub iteration: u32,
    pub format: AssetFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetVerdict {
    Accept { score: f64 },
    Refine { score: f64, feedback: String },
    Reject { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPipelineConfig {
    pub max_iterations: u32,
    pub score_threshold: f64,
    pub staging_dir: PathBuf,
}

impl Default for AssetPipelineConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            score_threshold: 70.0,
            staging_dir: PathBuf::from(".btc/staging/assets"),
        }
    }
}
