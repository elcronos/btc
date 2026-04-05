use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "btc",
    version,
    about = "Multi-agent orchestration CLI for Claude Code",
    long_about = "BTC — Build Things with Claude\n\n\
                  Full pipeline:  btc new \"description\"  (interview → plan → run)\n\
                  Step by step:   btc new --interview-only → btc plan → btc run\n\n\
                  Execution modes: --mode autopilot|ralph|ultrawork|deep-interview",
    after_help = "EXAMPLES:\n  \
                  btc new \"a todo app with React\"     # Full pipeline: interview → plan → run\n  \
                  btc new \"a game\" --interview-only   # Just the interview step\n  \
                  btc run --mode ralph                # Execute with ralph loop\n  \
                  btc run --mode ultrawork            # Execute with parallel agents\n  \
                  btc skills list                     # List available skills (local + OMC)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build something — runs interview → plan → run pipeline
    #[command(alias = "create", alias = "build")]
    New {
        /// Brief description of what to build
        description: String,

        /// Only run the interview step (don't auto-plan and run)
        #[arg(long)]
        interview_only: bool,

        /// Execution mode for the run step
        #[arg(short, long, value_enum)]
        mode: Option<ExecMode>,
    },

    /// Initialize BTC in the current project
    #[command(alias = "init")]
    Setup,

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

        /// Execution mode
        #[arg(short, long, value_enum)]
        mode: Option<ExecMode>,

        /// Enable debug mode (show raw NDJSON)
        #[arg(long)]
        debug: bool,
    },

    /// Open the TUI dashboard
    #[command(alias = "dash", alias = "tui")]
    Dashboard,

    /// Manage available skills
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
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

    /// Launch the multi-pane workspace (tmux)
    #[command(alias = "ws")]
    Workspace,
}

/// Execution mode — controls how Claude Code agents are orchestrated
#[derive(Debug, Clone, ValueEnum)]
pub enum ExecMode {
    /// Default sequential DAG execution
    Default,
    /// Autonomous end-to-end execution (OMC autopilot)
    Autopilot,
    /// Loop until completion with verification (OMC ralph)
    Ralph,
    /// Parallel high-throughput execution (OMC ultrawork)
    Ultrawork,
    /// Socratic deep interview before execution (OMC deep-interview)
    DeepInterview,
}

impl ExecMode {
    /// Get the Claude Code system prompt modifier for this mode
    pub fn system_prompt_suffix(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Autopilot => "\n\nYou are in AUTOPILOT mode. Work autonomously from start to finish. \
                Do not ask for confirmation — make decisions and implement fully. \
                Plan thoroughly, then execute all steps without pausing.",
            Self::Ralph => "\n\nYou are in RALPH mode (Recursive Adaptive Loop). \
                After completing each task, verify your work by running tests and checking output. \
                If verification fails, fix and retry. Loop until the task passes all checks. \
                Never mark a task complete without evidence it works.",
            Self::Ultrawork => "\n\nYou are in ULTRAWORK mode. Maximize throughput by working on \
                independent tasks in parallel. Start all non-dependent tasks simultaneously. \
                Be aggressive about parallelization while respecting dependencies.",
            Self::DeepInterview => "\n\nBefore implementing, conduct a deep analysis of the requirements. \
                Identify ambiguities, edge cases, and potential issues. Ask clarifying questions \
                in comments within the code. Then implement with full awareness of all concerns.",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Autopilot => "autopilot",
            Self::Ralph => "ralph",
            Self::Ultrawork => "ultrawork",
            Self::DeepInterview => "deep-interview",
        }
    }
}

impl std::fmt::Display for ExecMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Subcommand, Debug)]
pub enum SkillsAction {
    /// List all available skills
    List,
    /// Show details of a skill
    Show {
        /// Skill name
        name: String,
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
