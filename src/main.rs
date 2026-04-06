mod asset_pipeline;
mod cli;
mod claude;
mod config;
mod cron;
mod daemon;
mod dag;
mod error;
mod hooks;
mod interview;
mod notifications;
mod observer;
mod orchestrator;
mod remote;
mod sandbox;
mod setup;
mod skills;
mod state;
mod tui;
mod types;
mod visual_qa;
mod workspace;

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use cli::{Cli, Commands, SkillsAction};
use colored::Colorize;
use config::BtcConfig;
use tracing_subscriber::{fmt, EnvFilter};

const BTC_LOGO: &str = r#"
  ██████╗ ████████╗ ██████╗
  ██╔══██╗╚══██╔══╝██╔════╝
  ██████╔╝   ██║   ██║
  ██╔══██╗   ██║   ██║
  ██████╔╝   ██║   ╚██████╗
  ╚═════╝    ╚═╝    ╚═════╝
"#;

fn print_banner() {
    println!("{}", BTC_LOGO.cyan().bold());
    println!(
        "  {}  v{}\n",
        "Build Things with Claude".white().bold(),
        env!("CARGO_PKG_VERSION").dimmed()
    );
}

fn print_interactive_menu() {
    println!("{}", "━".repeat(52).dimmed());
    println!("  {}", "Workflow".white().bold());
    println!("{}", "─".repeat(52).dimmed());
    println!("  {:<18} {}", "/new <desc>".cyan(), "Full pipeline: interview → plan → run");
    println!("  {:<18} {}", "/interview <desc>".cyan(), "Interview only (no auto-run)");
    println!("  {:<18} {}", "/plan".cyan(), "Generate execution DAG from spec");
    println!("  {:<18} {}", "/run [mode]".cyan(), "Execute plan (default/ralph/ultrawork)");
    println!();
    println!("  {}", "Modes".white().bold());
    println!("{}", "─".repeat(52).dimmed());
    println!("  {:<18} {}", "default".dimmed(), "Sequential DAG execution");
    println!("  {:<18} {}", "autopilot".magenta(), "Autonomous end-to-end");
    println!("  {:<18} {}", "ralph".magenta(), "Loop with verification until done");
    println!("  {:<18} {}", "ultrawork".magenta(), "Parallel high-throughput");
    println!("  {:<18} {}", "deep-interview".magenta(), "Deep analysis before execution");
    println!();
    println!("  {}", "Observe".white().bold());
    println!("{}", "─".repeat(52).dimmed());
    println!("  {:<18} {}", "/status".cyan(), "Project overview (specs, plans, daemon)");
    println!("  {:<18} {}", "/dash".cyan(), "Live TUI dashboard (during /run)");
    println!("  {:<18} {}", "/skills".cyan(), "List skills (local + OMC + Claude)");
    println!();
    println!("  {}", "System".white().bold());
    println!("{}", "─".repeat(52).dimmed());
    println!("  {:<18} {}", "/workspace".cyan(), "Launch tmux multi-pane workspace");
    println!("  {:<18} {}", "/setup".cyan(), "Initialize BTC in current project");
    println!("  {:<18} {}", "/help".cyan(), "Show this menu");
    println!("  {:<18} {}", "/quit".cyan(), "Exit BTC");
    println!("{}", "━".repeat(52).dimmed());
}

async fn interactive_mode(project_dir: PathBuf) -> error::BtcResult<()> {
    print_banner();
    print_interactive_menu();
    println!();

    loop {
        print!("{} ", "btc ❯".cyan().bold());
        io::stdout().flush().ok();

        let mut line = String::new();
        io::stdin().read_line(&mut line).ok();
        let input = line.trim().to_string();

        if input.is_empty() {
            continue;
        }

        // Normalize: strip "btc " prefix for commands
        let normalized = if input.starts_with("btc ") {
            format!("/{}", &input[4..])
        } else {
            input.clone()
        };

        // Only treat as command if starts with /
        let is_command = normalized.starts_with('/');
        let parts: Vec<&str> = normalized.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("").trim();

        if !is_command {
            // Send to Claude as a question
            ask_claude(&project_dir, &input);
            println!();
            continue;
        }

        match cmd {
            "/quit" | "/exit" | "/q" => {
                println!("{}", "Goodbye!".cyan());
                break;
            }
            "/help" => {
                print_interactive_menu();
            }
            "/new" | "/create" | "/build" => {
                if arg.is_empty() {
                    println!("{} Usage: /new <description>", "!".yellow());
                } else {
                    // Full pipeline: interview → plan → run
                    let runner = interview::InterviewRunner::new(project_dir.clone());
                    match runner.run(arg) {
                        Ok(spec_path) => {
                            println!();
                            // Auto-plan
                            match run_plan(&project_dir, spec_path, false) {
                                Ok(_) => {
                                    println!();
                                    // Auto-run
                                    let plans_dir = project_dir.join(".btc").join("plans");
                                    if let Some(plan) = find_latest_file(&plans_dir, "plan-", ".json") {
                                        match run_execute(&project_dir, plan, false).await {
                                            Ok(_) => {}
                                            Err(e) => println!("{} Run failed: {}", "✗".red(), e),
                                        }
                                    }
                                }
                                Err(e) => println!("{} Plan failed: {}", "✗".red(), e),
                            }
                        }
                        Err(e) => println!("{} {}", "✗".red(), e),
                    }
                }
            }
            "/interview" => {
                // Interview only, no auto-chain
                if arg.is_empty() {
                    println!("{} Usage: /interview <description>", "!".yellow());
                } else {
                    let runner = interview::InterviewRunner::new(project_dir.clone());
                    match runner.run(arg) {
                        Ok(path) => println!("{} Spec: {}", "✓".green(), path.display()),
                        Err(e) => println!("{} {}", "✗".red(), e),
                    }
                }
            }
            "/plan" => {
                let specs_dir = project_dir.join(".btc").join("specs");
                match find_latest_spec(&specs_dir) {
                    Some(spec) => match run_plan(&project_dir, spec, false) {
                        Ok(_) => {}
                        Err(e) => println!("{} {}", "✗".red(), e),
                    },
                    None => println!("{} No spec found. Run /new first.", "!".yellow()),
                }
            }
            s if s.starts_with("/run") => {
                // Parse optional mode: /run, /run ralph, /run ultrawork, etc.
                let mode_str = s.strip_prefix("/run").unwrap_or("").trim();

                let mut mode = if mode_str.is_empty() {
                    // Show mode picker
                    println!("  {} Select execution mode:", "⚡".bold());
                    println!();
                    println!("  {}  {} — Sequential DAG execution", "1".cyan().bold(), "default");
                    println!("  {}  {} — Autonomous end-to-end", "2".cyan().bold(), "autopilot");
                    println!("  {}  {} — Loop with verification until done", "3".cyan().bold(), "ralph");
                    println!("  {}  {} — Parallel high-throughput", "4".cyan().bold(), "ultrawork");
                    println!("  {}  {} — Deep analysis before execution", "5".cyan().bold(), "deep-interview");
                    println!();
                    print!("  {} Choice [1-5]: ", "→".green().bold());
                    io::stdout().flush().ok();

                    let mut choice = String::new();
                    io::stdin().read_line(&mut choice).ok();
                    match choice.trim() {
                        "1" | "" => None,
                        "2" => Some(cli::ExecMode::Autopilot),
                        "3" => Some(cli::ExecMode::Ralph),
                        "4" => Some(cli::ExecMode::Ultrawork),
                        "5" => Some(cli::ExecMode::DeepInterview),
                        other => parse_exec_mode(other),
                    }
                } else {
                    parse_exec_mode(mode_str)
                };

                // Check if first run (no completed plans in state)
                let state_dir = project_dir.join(".btc").join("state");
                let is_first_run = !state_dir.join("last-run.json").exists();

                if is_first_run {
                    println!("  {} First run detected — running deep analysis before execution.", "ℹ".blue());
                    if mode.is_none() {
                        mode = Some(cli::ExecMode::DeepInterview);
                    }
                }

                if let Some(ref m) = mode {
                    println!("  {} Mode: {}", "⚡".bold(), m.label().cyan().bold());
                }

                let plans_dir = project_dir.join(".btc").join("plans");
                match find_latest_file(&plans_dir, "plan-", ".json") {
                    Some(plan) => {
                        match run_execute_with_mode(&project_dir, plan, false, mode.as_ref()).await {
                            Ok(_) => {
                                // Write first-run marker after successful execution
                                std::fs::create_dir_all(&state_dir).ok();
                                std::fs::write(
                                    state_dir.join("last-run.json"),
                                    format!("{{\"timestamp\":\"{}\"}}", chrono::Utc::now().to_rfc3339())
                                ).ok();

                                // Show event count if log exists
                                let event_log = project_dir.join(".btc").join("agent-events.jsonl");
                                if event_log.exists() {
                                    let line_count = std::fs::read_to_string(&event_log)
                                        .map(|c| c.lines().count())
                                        .unwrap_or(0);
                                    println!("  {} Events tracked: {}", "📊".dimmed(), line_count);
                                }
                            }
                            Err(e) => println!("{} {}", "✗".red(), e),
                        }
                    }
                    None => println!("{} No plan found. Run /plan first.", "!".yellow()),
                }
            }
            "/dash" | "/dashboard" | "/tui" => {
                match tui::run_dashboard(&project_dir) {
                    Ok(_) => {}
                    Err(e) => println!("{} Dashboard error: {}", "✗".red(), e),
                }
            }
            "/status" => match run_status(&project_dir) {
                Ok(_) => {}
                Err(e) => println!("{} {}", "✗".red(), e),
            },
            "/skills" => match run_skills_list(&project_dir) {
                Ok(_) => {}
                Err(e) => println!("{} {}", "✗".red(), e),
            },
            "/setup" | "/init" => {
                let wizard = setup::SetupWizard::new(project_dir.clone());
                match wizard.run() {
                    Ok(_) => {}
                    Err(e) => println!("{} {}", "✗".red(), e),
                }
            }
            "/workspace" | "/ws" => {
                match workspace::launch_workspace(&project_dir) {
                    Ok(_) => {}
                    Err(e) => println!("{} {}", "✗".red(), e),
                }
            }
            _ => {
                println!(
                    "{} Unknown command: {}. Type {} for help, or ask without {} to chat with Claude.",
                    "?".yellow(),
                    cmd.yellow(),
                    "/help".cyan(),
                    "/".yellow()
                );
            }
        }

        println!();
    }

    Ok(())
}

fn run_plan(project_dir: &Path, spec: PathBuf, consensus: bool) -> error::BtcResult<()> {
    use dag::planner::DagPlanner;

    let sep = "─".repeat(44);
    println!("{}", sep.dimmed());
    println!("{} {}", "●".cyan(), "Generating execution plan…".white());

    let spec_content = std::fs::read_to_string(&spec)?;
    let graph = if consensus {
        DagPlanner::from_consensus(&spec_content)?
    } else {
        DagPlanner::from_spec(&spec_content)?
    };

    let plans_dir = project_dir.join(".btc").join("plans");
    std::fs::create_dir_all(&plans_dir)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let plan_path = plans_dir.join(format!("plan-{}.json", timestamp));
    let json = serde_json::to_string_pretty(&graph)?;
    std::fs::write(&plan_path, json)?;

    println!(
        "{} Plan written: {}",
        "✓".green().bold(),
        plan_path.display().to_string().yellow()
    );
    println!("{} Node count: {}", "→".cyan(), graph.node_count());
    println!("{}", sep.dimmed());
    Ok(())
}

async fn run_execute(
    project_dir: &Path,
    plan: PathBuf,
    debug_mode: bool,
) -> error::BtcResult<()> {
    use dag::executor::DagExecutor;
    use dag::graph::TaskGraph;

    let sep = "─".repeat(44);
    println!("{}", sep.dimmed());
    println!("{} {}", "●".cyan(), "Executing plan…".white());

    let json = std::fs::read_to_string(&plan)?;
    let graph: TaskGraph = serde_json::from_str(&json)?;
    let mut executor = DagExecutor::new(graph, project_dir.to_path_buf()).with_debug(debug_mode);
    let report = executor.execute().await?;

    println!("{}", sep.dimmed());
    println!("{} Execution complete", "✓".green().bold());
    println!(
        "  Completed: {}  Failed: {}  Time: {:.1}s",
        report.completed_nodes.len().to_string().green(),
        if report.failed_nodes.is_empty() {
            "0".green().to_string()
        } else {
            report.failed_nodes.len().to_string().red().to_string()
        },
        report.elapsed_secs
    );
    println!("{}", sep.dimmed());
    Ok(())
}

fn run_status(project_dir: &Path) -> error::BtcResult<()> {
    let btc_dir = project_dir.join(".btc");
    let initialized = btc_dir.exists();

    println!("{}", "─".repeat(44).dimmed());
    println!("{} Project Status", "●".cyan());
    println!(
        "  Initialized : {}",
        if initialized {
            "yes".green().to_string()
        } else {
            "no".red().to_string()
        }
    );

    if initialized {
        let specs_dir = btc_dir.join("specs");
        let plans_dir = btc_dir.join("plans");
        let skills_dir = btc_dir.join("skills");

        let spec_count = count_files(&specs_dir, ".md");
        let plan_count = count_files(&plans_dir, ".json");
        let skill_count = count_files(&skills_dir, ".md");

        println!("  Specs       : {}", spec_count.to_string().yellow());
        println!("  Plans       : {}", plan_count.to_string().yellow());
        println!("  Skills      : {}", skill_count.to_string().yellow());

        let daemon_sock = btc_dir.join("daemon.sock");
        println!(
            "  Daemon      : {}",
            if daemon_sock.exists() {
                "running".green().to_string()
            } else {
                "offline".dimmed().to_string()
            }
        );
    }
    println!("{}", "─".repeat(44).dimmed());
    Ok(())
}

fn run_skills_list(project_dir: &Path) -> error::BtcResult<()> {
    let registry = skills::SkillRegistry::load_all(project_dir)?;
    let skill_list = registry.list();

    println!();
    println!("{}", "━".repeat(50).dimmed());
    println!(
        "  {} {}",
        "⚡".bold(),
        "Available Skills".bold().cyan()
    );
    println!("{}", "─".repeat(50).dimmed());

    if skill_list.is_empty() {
        println!("  {} No skills found.", "·".dimmed());
        println!(
            "  {} Add .md files to {}",
            "→".dimmed(),
            ".btc/skills/".yellow()
        );
    } else {
        let mut current_source = String::new();
        for skill in &skill_list {
            let source_label = match skill.source {
                skills::SkillSource::Local => "local",
                skills::SkillSource::OMC => "omc",
                skills::SkillSource::Claude => "claude",
            };
            let source_badge = match skill.source {
                skills::SkillSource::Local => format!("[{}]", "local".green()),
                skills::SkillSource::OMC => format!("[{}]", "omc".magenta()),
                skills::SkillSource::Claude => format!("[{}]", "claude".blue()),
            };

            if source_label != current_source {
                if !current_source.is_empty() {
                    println!();
                }
                current_source = source_label.to_string();
            }

            println!(
                "  {} {:<24} {} {}",
                "●".yellow(),
                skill.name.white().bold(),
                source_badge,
                skill.description.dimmed()
            );
        }
    }

    println!("{}", "━".repeat(50).dimmed());
    println!();

    Ok(())
}

fn ask_claude(project_dir: &Path, question: &str) {
    use std::process::{Command, Stdio};
    println!("  {} Asking Claude...", "●".yellow());

    let result = Command::new("claude")
        .arg("-p")
        .arg(question)
        .current_dir(project_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            println!();
            for line in text.trim().lines() {
                println!("  {}", line);
            }
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr);
            println!("  {} {}", "✗".red(), err.trim());
        }
        Err(e) => {
            println!("  {} Claude not available: {}", "✗".red(), e);
        }
    }
}

fn parse_exec_mode(s: &str) -> Option<cli::ExecMode> {
    match s.to_lowercase().as_str() {
        "" | "default" => None,
        "autopilot" | "auto" => Some(cli::ExecMode::Autopilot),
        "ralph" => Some(cli::ExecMode::Ralph),
        "ultrawork" | "ulw" => Some(cli::ExecMode::Ultrawork),
        "deep-interview" | "deep_interview" | "di" => Some(cli::ExecMode::DeepInterview),
        _ => {
            println!("  {} Unknown mode: {}. Using default.", "!".yellow(), s.red());
            None
        }
    }
}

async fn run_execute_with_mode(
    project_dir: &Path,
    plan: PathBuf,
    debug_mode: bool,
    mode: Option<&cli::ExecMode>,
) -> error::BtcResult<()> {
    use dag::executor::DagExecutor;
    use dag::graph::TaskGraph;

    let sep = "─".repeat(44);
    println!("{}", sep.dimmed());

    if let Some(m) = mode {
        println!("{} {} mode: {}", "⚡".bold(), "Executing in".white(), m.label().cyan().bold());
    } else {
        println!("{} {}", "●".cyan(), "Executing plan…".white());
    }

    let json = std::fs::read_to_string(&plan)?;
    let mut graph: TaskGraph = serde_json::from_str(&json)?;

    // Apply mode system prompt suffix to all node prompts
    if let Some(m) = mode {
        let suffix = m.system_prompt_suffix();
        if !suffix.is_empty() {
            for node in graph.graph.node_weights_mut() {
                node.prompt.push_str(suffix);
            }
        }
    }

    let mut executor = DagExecutor::new(graph, project_dir.to_path_buf()).with_debug(debug_mode);
    let report = executor.execute().await?;

    println!("{}", sep.dimmed());
    println!("{} Execution complete", "✓".green().bold());
    println!(
        "  Completed: {}  Failed: {}  Time: {:.1}s",
        report.completed_nodes.len().to_string().green(),
        if report.failed_nodes.is_empty() {
            "0".green().to_string()
        } else {
            report.failed_nodes.len().to_string().red().to_string()
        },
        report.elapsed_secs
    );
    println!("{}", sep.dimmed());
    Ok(())
}

fn find_latest_spec(specs_dir: &Path) -> Option<PathBuf> {
    // Spec files can be named interview-*.md or spec-*.md
    let mut all: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(specs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                all.push(path);
            }
        }
    }
    all.sort();
    all.pop()
}

fn find_latest_file(dir: &Path, prefix: &str, suffix: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(prefix) && n.ends_with(suffix))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files.pop()
}

fn count_files(dir: &Path, ext: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|x| x.to_str())
                        .map(|x| format!(".{}", x) == ext)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

#[tokio::main]
async fn main() -> error::BtcResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let project_dir = std::env::current_dir()?;

    if args.len() <= 1 {
        if std::env::var("TMUX").is_ok() {
            return interactive_mode(project_dir).await;
        }
        match workspace::launch_workspace(&project_dir) {
            Ok(_) => return Ok(()),
            Err(_) => return interactive_mode(project_dir).await,
        }
    }

    let cli = Cli::parse();

    let filter = if cli.verbose {
        EnvFilter::new("btc=debug")
    } else {
        EnvFilter::new("btc=info")
    };
    fmt().with_env_filter(filter).init();

    let _config = BtcConfig::load(&project_dir).unwrap_or_default();

    match cli.command {
        Commands::Setup => {
            print_banner();
            let wizard = setup::SetupWizard::new(project_dir);
            wizard.run()
        }
        Commands::New { description, interview_only, mode } => {
            let runner = interview::InterviewRunner::new(project_dir.clone());
            let spec_path = runner.run(&description)?;

            if interview_only {
                return Ok(());
            }

            // Auto-plan
            println!();
            run_plan(&project_dir, spec_path, false)?;

            // Auto-run
            println!();
            let plans_dir = project_dir.join(".btc").join("plans");
            match find_latest_file(&plans_dir, "plan-", ".json") {
                Some(plan) => run_execute_with_mode(&project_dir, plan, false, mode.as_ref()).await?,
                None => println!("{} No plan generated.", "!".yellow()),
            }

            Ok(())
        }
        Commands::Plan { spec, consensus } => {
            let specs_dir = project_dir.join(".btc").join("specs");
            let spec_path = match spec {
                Some(s) => PathBuf::from(s),
                None => match find_latest_spec(&specs_dir) {
                    Some(p) => p,
                    None => {
                        println!("{} No spec found. Run btc new first.", "!".yellow());
                        return Ok(());
                    }
                },
            };
            run_plan(&project_dir, spec_path, consensus)
        }
        Commands::Run { plan, mode, debug: debug_mode } => {
            let plans_dir = project_dir.join(".btc").join("plans");
            let plan_path = match plan {
                Some(p) => PathBuf::from(p),
                None => match find_latest_file(&plans_dir, "plan-", ".json") {
                    Some(p) => p,
                    None => {
                        println!("{} No plan found. Run btc plan first.", "!".yellow());
                        return Ok(());
                    }
                },
            };
            run_execute_with_mode(&project_dir, plan_path, debug_mode, mode.as_ref()).await
        }
        Commands::Dashboard => {
            tui::run_dashboard(&project_dir)
        }
        Commands::Skills { action } => match action {
            SkillsAction::List => run_skills_list(&project_dir),
            SkillsAction::Show { name } => {
                let registry = skills::SkillRegistry::load_all(&project_dir)?;
                match registry.get(&name) {
                    Some(skill) => {
                        let source_badge = match skill.source {
                            skills::SkillSource::Local => format!("[{}]", "local".green()),
                            skills::SkillSource::OMC => format!("[{}]", "omc".magenta()),
                            skills::SkillSource::Claude => format!("[{}]", "claude".blue()),
                        };
                        println!();
                        println!("  {} {} {}", "●".yellow(), skill.name.white().bold(), source_badge);
                        println!("  {}", skill.description.dimmed());
                        if !skill.triggers.is_empty() {
                            println!("  {} {}", "Triggers:".dimmed(), skill.triggers.join(", ").yellow());
                        }
                        println!();
                        println!("{}", "─".repeat(50).dimmed());
                        println!("{}", skill.content);
                        println!("{}", "─".repeat(50).dimmed());
                        println!();
                        Ok(())
                    }
                    None => {
                        eprintln!("  {} Skill not found: {}", "✗".red().bold(), name.red());
                        Err(error::BtcError::SkillNotFound(name))
                    }
                }
            }
        },
        Commands::Daemon { action } => {
            let btc_dir = project_dir.join(".btc");
            let socket_path = btc_dir.join("daemon.sock");
            match action {
                cli::DaemonAction::Start => {
                    let client = daemon::DaemonClient::new(socket_path.clone());
                    if client.is_daemon_running() {
                        println!("  {} Daemon is already running.", "ℹ".blue());
                        return Ok(());
                    }
                    std::fs::create_dir_all(&btc_dir)?;
                    println!("  {} Starting BTC daemon...", "●".yellow());
                    let server = daemon::DaemonServer::new(socket_path);
                    server.listen().await?;
                }
                cli::DaemonAction::Stop => {
                    if !socket_path.exists() {
                        println!("  {} Daemon is not running.", "·".dimmed());
                        return Ok(());
                    }
                    let _ = std::fs::remove_file(&socket_path);
                    println!("  {} Daemon stopped.", "✓".green().bold());
                }
                cli::DaemonAction::Status => {
                    let client = daemon::DaemonClient::new(socket_path);
                    if client.is_daemon_running() {
                        match client.send(daemon::DaemonCommand::Status).await {
                            Ok(response) => println!("  {} {:?}", "✓".green(), response),
                            Err(e) => println!("  {} {}", "✗".red(), e),
                        }
                    } else {
                        println!("  {} Daemon is not running.", "·".dimmed());
                    }
                }
                cli::DaemonAction::Setup => {
                    println!("  {} Setting up daemon for auto-start...", "●".yellow());
                    let binary = std::env::current_exe()?;
                    daemon::LaunchdManager::install(&binary)?;
                    println!("  {} Launchd service installed.", "✓".green().bold());
                }
            }
            Ok(())
        }
        Commands::Cron { action } => {
            let state_dir = project_dir.join(".btc").join("state").join("cron");
            std::fs::create_dir_all(&state_dir)?;
            let mut scheduler = cron::CronScheduler::new(state_dir.clone());
            scheduler.load_state()?;
            match action {
                cli::CronAction::Add { schedule, skill } => {
                    let id = scheduler.add_job(&schedule, &skill)?;
                    println!("  {} Cron job added: {}", "✓".green().bold(), id.cyan());
                    println!("    {} {}", "Schedule:".dimmed(), schedule);
                    println!("    {} {}", "Skill:".dimmed(), skill);
                }
                cli::CronAction::Remove { job_id } => {
                    scheduler.remove_job(&job_id)?;
                    println!("  {} Cron job removed: {}", "✓".green().bold(), job_id);
                }
                cli::CronAction::List => {
                    let jobs = scheduler.list_jobs();
                    if jobs.is_empty() {
                        println!("  {} No scheduled jobs.", "·".dimmed());
                    } else {
                        println!("  {:<12} {:<20} {:<15} {}", "ID".white().bold(), "Schedule".white().bold(), "Skill".white().bold(), "Enabled".white().bold());
                        println!("  {}", "─".repeat(55).dimmed());
                        for job in jobs {
                            println!("  {:<12} {:<20} {:<15} {}", job.id.cyan(), job.schedule, job.skill_name.yellow(), if job.enabled { "yes".green().to_string() } else { "no".red().to_string() });
                        }
                    }
                }
                cli::CronAction::Logs { job_id } => {
                    let log_path = state_dir.join(format!("{}.log", job_id));
                    if log_path.exists() {
                        print!("{}", std::fs::read_to_string(&log_path)?);
                    } else {
                        println!("  {} No logs found for job: {}", "·".dimmed(), job_id);
                    }
                }
            }
            Ok(())
        }
        Commands::Status => run_status(&project_dir),
        Commands::Workspace => {
            workspace::launch_workspace(&project_dir)
        }
        Commands::Debug { action } => {
            match action {
                cli::DebugAction::Dump => {
                    let state_dir = project_dir.join(".btc").join("state");
                    if let Ok(state_mgr) = state::StateManager::new(state_dir) {
                        if let Ok(status) = state_mgr.read::<serde_json::Value>("orchestrator") {
                            println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
                        } else {
                            println!("  {} No state found.", "·".dimmed());
                        }
                    }
                }
                cli::DebugAction::Stream => {
                    println!("  {} Run {} to see raw Claude Code events.", "ℹ".blue(), "btc run --debug".yellow());
                }
            }
            Ok(())
        }
    }
}
