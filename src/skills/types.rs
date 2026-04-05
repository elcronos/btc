use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub content: String,
    pub allowed_tools: Option<Vec<String>>,
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub source: SkillSource,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum SkillSource {
    #[default]
    Local,
    OMC,
    Claude,
}
