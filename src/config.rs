use crate::error::BtcResult;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcConfig {
    #[serde(default = "default_project_dir")]
    pub project_dir: PathBuf,

    #[serde(default)]
    pub visual_qa: VisualQaConfig,

    #[serde(default)]
    pub sandbox: SandboxConfig,

    #[serde(default)]
    pub remote: RemoteConfig,

    #[serde(default)]
    pub limits: LimitsConfig,
}

fn default_project_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualQaConfig {
    #[serde(default = "default_page_threshold")]
    pub page_score_threshold: f64,

    #[serde(default = "default_cross_page_threshold")]
    pub cross_page_threshold: f64,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_viewport")]
    pub viewport: (u32, u32),
}

impl Default for VisualQaConfig {
    fn default() -> Self {
        Self {
            page_score_threshold: 90.0,
            cross_page_threshold: 85.0,
            max_retries: 3,
            viewport: (1280, 720),
        }
    }
}

fn default_page_threshold() -> f64 { 90.0 }
fn default_cross_page_threshold() -> f64 { 85.0 }
fn default_max_retries() -> u32 { 3 }
fn default_viewport() -> (u32, u32) { (1280, 720) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub network_policy: NetworkPolicy,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            network_policy: NetworkPolicy::Permissive,
        }
    }
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkPolicy {
    Permissive,
    Strict,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self::Permissive
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub telegram: Option<TelegramConfig>,
    pub slack: Option<SlackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_chat_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub app_token: String,
    pub bot_token: String,
    pub allowed_workspace_ids: Vec<String>,
    pub allowed_user_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub max_concurrent_agents: Option<usize>,
    pub max_budget_usd: Option<f64>,
    pub max_iterations: Option<u32>,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: None,
            max_budget_usd: None,
            max_iterations: None,
        }
    }
}

impl BtcConfig {
    pub fn load(project_dir: &Path) -> BtcResult<Self> {
        let config_path = project_dir.join(".btc").join("config.toml");
        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let mut config: BtcConfig = toml::from_str(&contents)?;
            config.project_dir = project_dir.to_path_buf();
            Ok(config)
        } else {
            Ok(Self {
                project_dir: project_dir.to_path_buf(),
                ..Self::default()
            })
        }
    }

    pub fn btc_dir(&self) -> PathBuf {
        self.project_dir.join(".btc")
    }

    pub fn state_dir(&self) -> PathBuf {
        self.btc_dir().join("state")
    }

    pub fn specs_dir(&self) -> PathBuf {
        self.btc_dir().join("specs")
    }

    pub fn plans_dir(&self) -> PathBuf {
        self.btc_dir().join("plans")
    }

    pub fn skills_dir(&self) -> PathBuf {
        self.btc_dir().join("skills")
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.btc_dir().join("staging")
    }
}

impl Default for BtcConfig {
    fn default() -> Self {
        Self {
            project_dir: default_project_dir(),
            visual_qa: VisualQaConfig::default(),
            sandbox: SandboxConfig::default(),
            remote: RemoteConfig::default(),
            limits: LimitsConfig::default(),
        }
    }
}
