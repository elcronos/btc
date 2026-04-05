use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "btc", version, about = "Multi-agent orchestration CLI for Claude Code")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize BTC in the current project
    Setup,

    /// Start a deep interview to crystallize requirements
    New {
        /// Brief description of what to build
        description: String,
    },

    /// Generate an execution plan from a spec
    Plan {
        /// Spec file path (default: latest in .btc/specs/)
        #[arg(short, long)]
        spec: Option<String>,

        /// Use Planner/Architect/Critic consensus
        #[arg(long)]
        consensus: bool,
    },

    /// Execute a plan with multi-agent orchestration
    Run {
        /// Plan file path (default: latest in .btc/plans/)
        #[arg(short, long)]
        plan: Option<String>,

        /// Enable debug mode (show raw NDJSON)
        #[arg(long)]
        debug: bool,
    },

    /// Manage the background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Manage scheduled skill workflows
    Cron {
        #[command(subcommand)]
        action: CronAction,
    },

    /// Show current status
    Status,

    /// Debug utilities
    Debug {
        #[command(subcommand)]
        action: DebugAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonAction {
    /// Start the background daemon
    Start,
    /// Stop the background daemon
    Stop,
    /// Show daemon status
    Status,
    /// Configure remote control channels
    Setup,
}

#[derive(Subcommand, Debug)]
pub enum CronAction {
    /// Add a scheduled skill workflow
    Add {
        /// Cron schedule expression (e.g., "0 2 * * *")
        schedule: String,
        /// Skill name to execute
        skill: String,
    },
    /// Remove a scheduled workflow
    Remove {
        /// Job ID to remove
        job_id: String,
    },
    /// List all scheduled workflows
    List,
    /// Show logs for a cron job
    Logs {
        /// Job ID
        job_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DebugAction {
    /// Dump full state to stdout
    Dump,
    /// Show raw Claude Code stream events
    Stream,
}
