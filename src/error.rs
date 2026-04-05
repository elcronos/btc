use std::path::PathBuf;
use thiserror::Error;

pub type BtcResult<T> = Result<T, BtcError>;

#[derive(Error, Debug)]
pub enum BtcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Claude Code not found. Run `btc setup` to configure.")]
    ClaudeCodeNotFound,

    #[error("Claude Code subprocess failed: {0}")]
    SubprocessFailed(String),

    #[error("Stream parse error: {0}")]
    StreamParse(String),

    #[error("Agent error: {agent_id} — {message}")]
    Agent { agent_id: String, message: String },

    #[error("Sandbox violation: path {path} is outside project directory")]
    SandboxViolation { path: PathBuf },

    #[error("Sandbox error: {0}")]
    Sandbox(String),

    #[error("Gitignore safety check failed: .btc/ is not in .gitignore")]
    GitignoreNotConfigured,

    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    #[error("Visual QA error: {0}")]
    VisualQa(String),

    #[error("DAG error: {0}")]
    Dag(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Hook error: {0}")]
    Hook(String),

    #[error("Daemon error: {0}")]
    Daemon(String),

    #[error("Remote control error: {0}")]
    Remote(String),

    #[error("Cron error: {0}")]
    Cron(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Browser not available: {0}")]
    BrowserUnavailable(String),

    #[error("Asset pipeline error: {0}")]
    AssetPipeline(String),

    #[error("{0}")]
    Other(String),
}
