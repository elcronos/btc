use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};

use crate::error::BtcResult;
use crate::observer::{self, AgentTrackStatus, TrackedAgent};

// ─── Tab Enum ────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Overview,
    Claude,
    Observability,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Overview => " 1 Overview ",
            Tab::Claude => " 2 Claude ",
            Tab::Observability => " 3 Observability ",
        }
    }
}

// ─── Process Scanner ─────────────────────────────────────

#[derive(Debug, Clone)]
struct ProcessAgent {
    pid: String,
    name: String,
    model: String,
    status: &'static str,
    duration: String,
    project: String,
    is_subagent: bool,
}

fn scan_all_claude_processes() -> Vec<ProcessAgent> {
    let output = Command::new("ps")
        .args(["-eo", "pid,etime,command"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut agents = Vec::new();

    for line in stdout.lines().skip(1) {
        let line = line.trim();
        if !line.contains("claude") {
            continue;
        }
        // Skip noise
        if line.contains("grep") || line.contains("ps -eo") || line.contains("btc") || line.contains("codesign") {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, char::is_whitespace).collect();
        if parts.len() < 3 {
            continue;
        }

        let pid = parts[0].trim().to_string();
        let etime = parts[1].trim().to_string();
        let cmd = parts[2].trim();

        // Detect model
        let model = if cmd.contains("--model") {
            cmd.split("--model")
                .nth(1)
                .and_then(|s| s.trim().split_whitespace().next())
                .unwrap_or("opus")
                .to_string()
        } else {
            "default".to_string()
        };

        // Detect project directory
        let project = extract_cwd(cmd).unwrap_or_else(|| "unknown".to_string());

        // Detect if subagent (has parent_tool_use_id or agent-specific flags)
        let is_subagent = cmd.contains("subagent") || cmd.contains("--agent");

        // Extract a friendly name
        let name = if cmd.contains("-p ") {
            let prompt = extract_flag_value(cmd, "-p ");
            let truncated = if prompt.len() > 40 {
                format!("{}...", &prompt[..37])
            } else {
                prompt
            };
            truncated
        } else if cmd.contains("claude ") && !cmd.contains("-p ") {
            "Interactive Session".to_string()
        } else {
            "Claude Agent".to_string()
        };

        agents.push(ProcessAgent {
            pid,
            name,
            model,
            status: "RUNNING",
            duration: etime,
            project,
            is_subagent,
        });
    }

    agents
}

fn extract_flag_value(cmd: &str, flag: &str) -> String {
    let idx = match cmd.find(flag) {
        Some(i) => i + flag.len(),
        None => return String::new(),
    };
    let after = cmd[idx..].trim_start();

    if after.starts_with('"') {
        let rest = &after[1..];
        let end = rest.find('"').unwrap_or(rest.len());
        rest[..end].to_string()
    } else if after.starts_with('\'') {
        let rest = &after[1..];
        let end = rest.find('\'').unwrap_or(rest.len());
        rest[..end].to_string()
    } else {
        let end = after.find(" -").unwrap_or(after.len());
        after[..end].trim().to_string()
    }
}

fn extract_cwd(cmd: &str) -> Option<String> {
    // Try to extract from the command path or flags
    if cmd.contains("--cwd ") || cmd.contains("-c ") {
        return Some(extract_flag_value(cmd, "--cwd "));
    }
    None
}

// ─── App State ───────────────────────────────────────────

struct DashApp {
    tab: Tab,
    project_dir: PathBuf,
    project_name: String,
    // Overview
    spec_count: usize,
    plan_count: usize,
    skill_count: usize,
    daemon_running: bool,
    cmd_input: String,
    cmd_output: Vec<String>,
    // Observability
    process_agents: Vec<ProcessAgent>,
    tracked_agents: Vec<TrackedAgent>,
    total_events: usize,
    tool_uses: usize,
    dag_completed: u32,
    dag_total: u32,
    // Claude launch flag
    launch_claude: bool,
}

impl DashApp {
    fn new(project_dir: PathBuf) -> Self {
        let project_name = project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        let mut app = Self {
            tab: Tab::Overview,
            project_dir,
            project_name,
            spec_count: 0,
            plan_count: 0,
            skill_count: 0,
            daemon_running: false,
            cmd_input: String::new(),
            cmd_output: Vec::new(),
            process_agents: Vec::new(),
            tracked_agents: Vec::new(),
            total_events: 0,
            tool_uses: 0,
            dag_completed: 0,
            dag_total: 0,
            launch_claude: false,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        let btc_dir = self.project_dir.join(".btc");

        self.spec_count = count_files_with_ext(&btc_dir.join("specs"), "md");
        self.plan_count = count_files_with_ext(&btc_dir.join("plans"), "json");
        self.skill_count = count_files_with_ext(&btc_dir.join("skills"), "md");
        self.daemon_running = btc_dir.join("daemon.sock").exists();

        // Scan system-wide processes
        self.process_agents = scan_all_claude_processes();

        // Also read JSONL events for this project
        self.tracked_agents = observer::read_agent_topology(&self.project_dir);
        let (events, tools) = observer::estimate_usage(&self.project_dir);
        self.total_events = events;
        self.tool_uses = tools;

        let (c, t) = read_dag_state(&self.project_dir);
        self.dag_completed = c;
        self.dag_total = t;
    }
}

// ─── Helpers ─────────────────────────────────────────────

fn count_files_with_ext(dir: &Path, ext: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some(ext))
                .count()
        })
        .unwrap_or(0)
}

fn read_dag_state(project_dir: &Path) -> (u32, u32) {
    let plans_dir = project_dir.join(".btc").join("plans");
    let entries = match std::fs::read_dir(&plans_dir) {
        Ok(e) => e,
        Err(_) => return (0, 0),
    };

    let mut latest: Option<PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            latest = Some(path);
        }
    }

    if let Some(path) = latest {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(graph) = serde_json::from_str::<crate::dag::graph::TaskGraph>(&content) {
                return (graph.completed_count() as u32, graph.node_count() as u32);
            }
        }
    }
    (0, 0)
}

// ─── Render: Overview Tab ────────────────────────────────

fn render_overview(frame: &mut Frame, area: Rect, app: &DashApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Status
            Constraint::Min(1),    // Output / activity
            Constraint::Length(3), // Input
        ])
        .split(area);

    // Project status
    let running_count = app.process_agents.len();
    let info_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Project  ", Style::default().fg(Color::Gray)),
            Span::styled(&app.project_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", Style::default().fg(Color::Gray)),
            Span::styled("Specs ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", app.spec_count), Style::default().fg(if app.spec_count > 0 { Color::Green } else { Color::DarkGray })),
            Span::styled("  │  ", Style::default().fg(Color::Gray)),
            Span::styled("Plans ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", app.plan_count), Style::default().fg(if app.plan_count > 0 { Color::Green } else { Color::DarkGray })),
            Span::styled("  │  ", Style::default().fg(Color::Gray)),
            Span::styled("Skills ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", app.skill_count), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Agents   ", Style::default().fg(Color::Gray)),
            Span::styled(format!("● {} running", running_count), Style::default().fg(if running_count > 0 { Color::Green } else { Color::DarkGray })),
            Span::styled("  │  ", Style::default().fg(Color::Gray)),
            Span::styled("Events ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", app.total_events), Style::default().fg(Color::Yellow)),
            Span::styled("  │  ", Style::default().fg(Color::Gray)),
            Span::styled("Tools ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", app.tool_uses), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Commands: /new <desc> │ /plan │ /run [mode] │ /skills │ /status │ /quit",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "  Modes:   default │ autopilot │ ralph │ ultrawork │ deep-interview",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "  Press 2 for Claude │ 3 for Observability │ Type commands below",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
    ];

    let info = Paragraph::new(Text::from(info_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ⚡ BTC Overview ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Gray)),
        );
    frame.render_widget(info, chunks[0]);

    // Output area
    let mut output_lines: Vec<Line> = vec![Line::from("")];
    if app.cmd_output.is_empty() {
        output_lines.push(Line::from(Span::styled(
            "  Ready. Type a command below or press 2 to open Claude.",
            Style::default().fg(Color::Gray),
        )));
    } else {
        for line in app.cmd_output.iter().rev().take(20).rev() {
            output_lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(Color::White),
            )));
        }
    }

    let output = Paragraph::new(Text::from(output_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Output ")
                .border_style(Style::default().fg(Color::Gray)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(output, chunks[1]);

    // Input
    let input = Paragraph::new(Span::styled(
        format!(" btc ❯ {}", app.cmd_input),
        Style::default().fg(Color::Cyan),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(input, chunks[2]);
}

// ─── Render: Claude placeholder (shows instructions) ─────

fn render_claude_placeholder(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "         Press Enter to launch Claude Code",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "         Claude opens in a split pane alongside this dashboard.",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "         Use Ctrl+B + arrow keys to switch between panes.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "         Project: current directory",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "         Mode: bypassPermissions (full tool access)",
            Style::default().fg(Color::Gray),
        )),
    ];

    let p = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Claude Code ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Gray)),
        );
    frame.render_widget(p, area);
}

// ─── Render: Observability Tab ───────────────────────────

fn render_observability(frame: &mut Frame, area: Rect, app: &DashApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Progress
            Constraint::Min(1),    // Agent topology
            Constraint::Length(5), // Stats
        ])
        .split(area);

    // DAG progress
    let ratio = if app.dag_total > 0 {
        (app.dag_completed as f64) / (app.dag_total as f64)
    } else {
        0.0
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" DAG Progress ")
                .border_style(Style::default().fg(Color::Gray)),
        )
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(ratio.min(1.0))
        .label(format!("{}/{} tasks", app.dag_completed, app.dag_total));
    frame.render_widget(gauge, chunks[0]);

    // Agent topology — merge process scan + JSONL events
    let mut lines: Vec<Line> = vec![Line::from("")];

    if app.process_agents.is_empty() && app.tracked_agents.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No Claude agents running on this machine.",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Start a task with:  btc run --mode ralph",
            Style::default().fg(Color::Gray),
        )));
    } else {
        // Live processes (system-wide)
        if !app.process_agents.is_empty() {
            lines.push(Line::from(Span::styled(
                "  LIVE AGENTS (system-wide)",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "  ─────────────────────────────────────────────────────────",
                Style::default().fg(Color::Gray),
            )));

            let total_live = app.process_agents.len();
            if total_live > 1 {
                lines.push(Line::from(vec![
                    Span::styled("  ⚡ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} agents running in parallel", total_live),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }
            lines.push(Line::from(""));

            for (i, agent) in app.process_agents.iter().enumerate() {
                let is_last = i == app.process_agents.len() - 1;
                let prefix = if is_last { "  └──" } else { "  ├──" };

                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(" ● ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format!("{:<30}", agent.name),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!(" model:{}", agent.model),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        format!("  pid:{}", agent.pid),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("  ⏱ {}", agent.duration),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }
        }

        // JSONL-tracked agents (historical + current project)
        if !app.tracked_agents.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  TRACKED AGENTS (this project)",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "  ─────────────────────────────────────────────────────────",
                Style::default().fg(Color::Gray),
            )));

            let running: Vec<&TrackedAgent> = app
                .tracked_agents
                .iter()
                .filter(|a| a.status == AgentTrackStatus::Running)
                .collect();
            let completed: Vec<&TrackedAgent> = app
                .tracked_agents
                .iter()
                .filter(|a| a.status == AgentTrackStatus::Completed)
                .collect();

            // Show waiting indicator
            if running.len() > 1 {
                lines.push(Line::from(vec![
                    Span::styled("  ⏳ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("Main agent dispatched {} parallel subagents", running.len()),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
                lines.push(Line::from(""));
            }

            for (i, agent) in app.tracked_agents.iter().enumerate() {
                let is_last = i == app.tracked_agents.len() - 1;
                let prefix = if is_last { "  └──" } else { "  ├──" };

                let (icon, color, label) = match agent.status {
                    AgentTrackStatus::Running => ("●", Color::Green, "RUN"),
                    AgentTrackStatus::Completed => ("✓", Color::Blue, "OK "),
                };

                let last_tool = agent
                    .tools_used
                    .last()
                    .map(|t| format!(" → {}", t))
                    .unwrap_or_default();

                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                    Span::styled(
                        format!("{:<22}", agent.agent_type),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("[{}]", label), Style::default().fg(color)),
                    Span::styled(
                        format!("  calls: {}", agent.tools_used.len()),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(last_tool, Style::default().fg(Color::Yellow)),
                ]));
            }
        }
    }

    let topology = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Agent Topology ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Gray)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(topology, chunks[1]);

    // Stats
    let running = app.process_agents.len();
    let tracked = app.tracked_agents.len();
    let tracked_done = app.tracked_agents.iter().filter(|a| a.status == AgentTrackStatus::Completed).count();
    let stats_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Live ", Style::default().fg(Color::Gray)),
            Span::styled(format!("● {}", running), Style::default().fg(Color::Green)),
            Span::styled("  │  Tracked ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{} total", tracked), Style::default().fg(Color::White)),
            Span::styled(format!(" ({} done)", tracked_done), Style::default().fg(Color::Blue)),
            Span::styled("  │  Events ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", app.total_events), Style::default().fg(Color::Yellow)),
            Span::styled("  │  Tools ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", app.tool_uses), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
    ];

    let stats = Paragraph::new(Text::from(stats_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Stats ")
                .border_style(Style::default().fg(Color::Gray)),
        );
    frame.render_widget(stats, chunks[2]);
}

// ─── Main Dashboard Loop ────────────────────────────────

pub fn run_dashboard(project_dir: &Path) -> BtcResult<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = DashApp::new(project_dir.to_path_buf());

    loop {
        app.refresh();

        // Check if we need to launch Claude
        if app.launch_claude {
            app.launch_claude = false;

            let in_tmux = std::env::var("TMUX").is_ok();
            let dir = app.project_dir.display().to_string();

            if in_tmux {
                // Split current tmux pane and run Claude there
                let _ = Command::new("tmux")
                    .args([
                        "split-window", "-h", "-p", "55",
                        "-c", &dir,
                        "claude", "--permission-mode", "bypassPermissions",
                    ])
                    .status();
                // Stay in the dashboard, switch to observability to monitor
                app.tab = Tab::Observability;
            } else {
                // Not in tmux — create a tmux session with both
                let session = format!("btc-{}", app.project_name);
                let _ = Command::new("tmux")
                    .args(["new-session", "-d", "-s", &session, "-c", &dir,
                           "btc"])  // left pane: dashboard
                    .status();
                let _ = Command::new("tmux")
                    .args(["split-window", "-t", &session, "-h", "-p", "55",
                           "-c", &dir,
                           "claude", "--permission-mode", "bypassPermissions"])
                    .status();
                let _ = Command::new("tmux")
                    .args(["select-pane", "-t", &format!("{}:0.0", session)])
                    .status();

                // Exit current dashboard and attach to the tmux session
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                terminal.show_cursor()?;
                let _ = Command::new("tmux")
                    .args(["attach-session", "-t", &session])
                    .status();
                return Ok(());
            }
            continue;
        }

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Tab bar
                    Constraint::Min(1),   // Content
                    Constraint::Length(1), // Status
                ])
                .split(frame.area());

            // Tab bar
            let tab_titles = vec![
                Line::from(Tab::Overview.label()),
                Line::from(Tab::Claude.label()),
                Line::from(Tab::Observability.label()),
            ];
            let tab_index = match app.tab {
                Tab::Overview => 0,
                Tab::Claude => 1,
                Tab::Observability => 2,
            };
            let tabs_widget = ratatui::widgets::Tabs::new(tab_titles)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" ⚡ BTC — {} ", app.project_name))
                        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                        .border_style(Style::default().fg(Color::Gray)),
                )
                .select(tab_index)
                .style(Style::default().fg(Color::Gray))
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                )
                .divider("│");
            frame.render_widget(tabs_widget, chunks[0]);

            // Content
            match app.tab {
                Tab::Overview => render_overview(frame, chunks[1], &app),
                Tab::Claude => render_claude_placeholder(frame, chunks[1]),
                Tab::Observability => render_observability(frame, chunks[1], &app),
            }

            // Status line
            let status = Line::from(vec![
                Span::styled(
                    format!(
                        " Live: {}  Events: {}  Tools: {} ",
                        app.process_agents.len(),
                        app.total_events,
                        app.tool_uses,
                    ),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled("│ ", Style::default().fg(Color::Gray)),
                Span::styled(
                    match app.tab {
                        Tab::Overview => "Type command + Enter │ 1-3: tabs │ q: quit",
                        Tab::Claude => "Enter: launch Claude │ 1-3: tabs │ Esc: back",
                        Tab::Observability => "Auto-refreshes every 2s │ 1-3: tabs │ q: quit",
                    },
                    Style::default().fg(Color::Gray),
                ),
            ]);
            frame.render_widget(Paragraph::new(status), chunks[2]);
        })?;

        // Input handling
        let timeout = if app.tab == Tab::Overview {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(2)
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Overview tab: typing commands
                if app.tab == Tab::Overview {
                    match key.code {
                        KeyCode::Char('q') if app.cmd_input.is_empty() => break,
                        KeyCode::Esc => break,
                        KeyCode::Char('1') if app.cmd_input.is_empty() => app.tab = Tab::Overview,
                        KeyCode::Char('2') if app.cmd_input.is_empty() => app.tab = Tab::Claude,
                        KeyCode::Char('3') if app.cmd_input.is_empty() => app.tab = Tab::Observability,
                        KeyCode::Char(c) => app.cmd_input.push(c),
                        KeyCode::Backspace => { app.cmd_input.pop(); }
                        KeyCode::Enter => {
                            if !app.cmd_input.is_empty() {
                                let cmd = app.cmd_input.clone();
                                app.cmd_input.clear();
                                app.cmd_output.push(format!("btc ❯ {}", cmd));

                                // Execute command
                                disable_raw_mode()?;
                                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                                let output = Command::new("btc")
                                    .args(cmd.split_whitespace())
                                    .current_dir(&app.project_dir)
                                    .output();

                                execute!(io::stdout(), EnterAlternateScreen)?;
                                enable_raw_mode()?;
                                terminal.clear()?;

                                match output {
                                    Ok(o) => {
                                        let stdout = String::from_utf8_lossy(&o.stdout);
                                        let stderr = String::from_utf8_lossy(&o.stderr);
                                        for line in stdout.lines() {
                                            app.cmd_output.push(line.to_string());
                                        }
                                        for line in stderr.lines() {
                                            app.cmd_output.push(line.to_string());
                                        }
                                    }
                                    Err(e) => {
                                        app.cmd_output.push(format!("Error: {}", e));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Claude tab
                if app.tab == Tab::Claude {
                    match key.code {
                        KeyCode::Enter => {
                            app.launch_claude = true;
                        }
                        KeyCode::Esc => app.tab = Tab::Overview,
                        KeyCode::Char('1') => app.tab = Tab::Overview,
                        KeyCode::Char('3') => app.tab = Tab::Observability,
                        _ => {}
                    }
                    continue;
                }

                // Observability tab
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('1') => app.tab = Tab::Overview,
                    KeyCode::Char('2') => app.tab = Tab::Claude,
                    KeyCode::Char('3') => app.tab = Tab::Observability,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
