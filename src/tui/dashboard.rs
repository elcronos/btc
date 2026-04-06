use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
    // Claude
    claude_input: String,
    claude_history: Vec<(String, String)>, // (question, answer)
    claude_loading: bool,
    // Observability
    agents: Vec<TrackedAgent>,
    total_events: usize,
    tool_uses: usize,
    dag_completed: u32,
    dag_total: u32,
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
            claude_input: String::new(),
            claude_history: Vec::new(),
            claude_loading: false,
            agents: Vec::new(),
            total_events: 0,
            tool_uses: 0,
            dag_completed: 0,
            dag_total: 0,
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

        self.agents = observer::read_agent_topology(&self.project_dir);
        let (events, tools) = observer::estimate_usage(&self.project_dir);
        self.total_events = events;
        self.tool_uses = tools;

        // Read DAG state
        let (c, t) = read_dag_state(&self.project_dir);
        self.dag_completed = c;
        self.dag_total = t;
    }

    fn running_agents(&self) -> usize {
        self.agents
            .iter()
            .filter(|a| a.status == AgentTrackStatus::Running)
            .count()
    }

    fn completed_agents(&self) -> usize {
        self.agents
            .iter()
            .filter(|a| a.status == AgentTrackStatus::Completed)
            .count()
    }
}

// ─── Helpers ─────────────────────────────────────────────

fn count_files_with_ext(dir: &Path, ext: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|x| x.to_str())
                        == Some(ext)
                })
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
            match (&latest, entry.metadata().and_then(|m| m.modified())) {
                (None, Ok(_)) => latest = Some(path),
                (Some(_), Ok(_)) => latest = Some(path),
                _ => {}
            }
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

fn ask_claude_sync(project_dir: &Path, question: &str) -> String {
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
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        Ok(output) => {
            format!(
                "Error: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
        }
        Err(e) => format!("Claude not available: {}", e),
    }
}

// ─── Render: Overview Tab ────────────────────────────────

fn render_overview(frame: &mut Frame, area: Rect, app: &DashApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Project info
            Constraint::Length(8),  // Quick commands
            Constraint::Min(1),    // Activity
        ])
        .split(area);

    // Project info box
    let mut info_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Project    ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.project_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Specs      ", Style::default().fg(Color::DarkGray)),
            if app.spec_count > 0 {
                Span::styled(format!("{} available", app.spec_count), Style::default().fg(Color::Green))
            } else {
                Span::styled("none — use /new", Style::default().fg(Color::DarkGray))
            },
        ]),
        Line::from(vec![
            Span::styled("  Plans      ", Style::default().fg(Color::DarkGray)),
            if app.plan_count > 0 {
                Span::styled(format!("{} available", app.plan_count), Style::default().fg(Color::Green))
            } else {
                Span::styled("none — use /plan", Style::default().fg(Color::DarkGray))
            },
        ]),
        Line::from(vec![
            Span::styled("  Skills     ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.skill_count), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Daemon     ", Style::default().fg(Color::DarkGray)),
            if app.daemon_running {
                Span::styled("● running", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ offline", Style::default().fg(Color::DarkGray))
            },
        ]),
        Line::from(vec![
            Span::styled("  Agents     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} running, {} completed", app.running_agents(), app.completed_agents()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
    ];

    let info = Paragraph::new(Text::from(info_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Project Status ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(info, chunks[0]);

    // Quick commands
    let cmd_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("2", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" to open Claude chat  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("3", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" for Agent Observability", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  CLI: ", Style::default().fg(Color::DarkGray)),
            Span::styled("btc new \"desc\"", Style::default().fg(Color::Yellow)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("btc plan", Style::default().fg(Color::Yellow)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("btc run --mode ralph", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Modes: ", Style::default().fg(Color::DarkGray)),
            Span::styled("default", Style::default().fg(Color::White)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("autopilot", Style::default().fg(Color::Magenta)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("ralph", Style::default().fg(Color::Magenta)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("ultrawork", Style::default().fg(Color::Magenta)),
        ]),
        Line::from(""),
    ];

    let cmds = Paragraph::new(Text::from(cmd_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Quick Reference ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(cmds, chunks[1]);

    // Activity / recent events
    let event_info = if app.total_events > 0 {
        format!(
            "  {} total events  │  {} tool calls  │  {} agents tracked",
            app.total_events, app.tool_uses, app.agents.len()
        )
    } else {
        "  No activity yet. Run a command to start.".to_string()
    };

    let activity = Paragraph::new(vec![Line::from(""), Line::from(Span::styled(event_info, Style::default().fg(Color::DarkGray)))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Activity ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(activity, chunks[2]);
}

// ─── Render: Claude Tab ──────────────────────────────────

fn render_claude(frame: &mut Frame, area: Rect, app: &DashApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Chat history
            Constraint::Length(3), // Input
        ])
        .split(area);

    // Chat history
    let mut lines: Vec<Line> = vec![Line::from("")];

    if app.claude_history.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Type a question below and press Enter to ask Claude.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Claude runs with full permissions in this project directory.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    for (q, a) in &app.claude_history {
        lines.push(Line::from(vec![
            Span::styled("  ❯ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(q.as_str(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        for answer_line in a.lines() {
            lines.push(Line::from(Span::styled(
                format!("    {}", answer_line),
                Style::default().fg(Color::White),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
    }

    if app.claude_loading {
        lines.push(Line::from(Span::styled(
            "  ● Thinking...",
            Style::default().fg(Color::Yellow),
        )));
    }

    let history = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Claude Chat ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false })
        .scroll((
            // Auto-scroll to bottom
            if app.claude_history.len() > 3 {
                ((app.claude_history.len() as u16).saturating_sub(2)) * 8
            } else {
                0
            },
            0,
        ));
    frame.render_widget(history, chunks[0]);

    // Input line
    let input_text = if app.claude_loading {
        " Waiting for Claude...".to_string()
    } else {
        format!(" ❯ {}", app.claude_input)
    };

    let input_style = if app.claude_loading {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let input = Paragraph::new(Span::styled(input_text, input_style))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Ask Claude (Enter to send, Esc to go back) ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(input, chunks[1]);
}

// ─── Render: Observability Tab ───────────────────────────

fn render_observability(frame: &mut Frame, area: Rect, app: &DashApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // DAG progress bar
            Constraint::Min(1),    // Agent topology
            Constraint::Length(6), // Stats panel
        ])
        .split(area);

    // DAG progress gauge
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
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(ratio.min(1.0))
        .label(format!(
            "{}/{} tasks complete",
            app.dag_completed, app.dag_total
        ));
    frame.render_widget(gauge, chunks[0]);

    // Agent topology with ASCII art
    let mut lines: Vec<Line> = vec![Line::from("")];

    if app.agents.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No agents detected. Waiting for activity...",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ┌─────────────────────────────────────┐",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "  │         (no active agents)          │",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "  └─────────────────────────────────────┘",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Separate main agent(s) from subagents
        let main_agents: Vec<&TrackedAgent> = app
            .agents
            .iter()
            .filter(|a| a.agent_type == "main")
            .collect();
        let sub_agents: Vec<&TrackedAgent> = app
            .agents
            .iter()
            .filter(|a| a.agent_type != "main")
            .collect();

        let running_subs: Vec<&&TrackedAgent> = sub_agents
            .iter()
            .filter(|a| a.status == AgentTrackStatus::Running)
            .collect();
        let completed_subs: Vec<&&TrackedAgent> = sub_agents
            .iter()
            .filter(|a| a.status == AgentTrackStatus::Completed)
            .collect();

        // Main agent box
        for main in &main_agents {
            let status = match main.status {
                AgentTrackStatus::Running => ("●", Color::Green, "RUNNING"),
                AgentTrackStatus::Completed => ("✓", Color::Blue, "DONE"),
            };
            let tool_count = main.tools_used.len();
            let last_tool = main
                .tools_used
                .last()
                .map(|t| t.as_str())
                .unwrap_or("idle");

            lines.push(Line::from(vec![
                Span::styled("  ┌─", Style::default().fg(Color::Cyan)),
                Span::styled("─────────────────────────────────────────────────────", Style::default().fg(Color::Cyan)),
                Span::styled("─┐", Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  │ ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{} ", status.0), Style::default().fg(status.1)),
                Span::styled(
                    format!("Main Agent  [{}]", status.2),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  tools: {}  last: {}", tool_count, last_tool),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(""),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  └─", Style::default().fg(Color::Cyan)),
                Span::styled("───────────┬─────────────────────────────────────────", Style::default().fg(Color::Cyan)),
                Span::styled("─┘", Style::default().fg(Color::Cyan)),
            ]));
        }

        if main_agents.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (no main agent detected)",
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Subagent tree
        if !sub_agents.is_empty() {
            let waiting = !running_subs.is_empty();

            if waiting && !main_agents.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("              │", Style::default().fg(Color::Cyan)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("              │  ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("⏳ Main agent waiting for {} subagent(s)", running_subs.len()),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("              │", Style::default().fg(Color::Cyan)),
                ]));
            }

            for (i, agent) in sub_agents.iter().enumerate() {
                let is_last = i == sub_agents.len() - 1;
                let connector = if is_last { "└──" } else { "├──" };
                let continuation = if is_last { "   " } else { "│  " };

                let (icon, color, status_label) = match agent.status {
                    AgentTrackStatus::Running => ("●", Color::Green, "RUN"),
                    AgentTrackStatus::Completed => ("✓", Color::Blue, "OK "),
                };

                let tool_summary = if agent.tools_used.is_empty() {
                    String::new()
                } else {
                    let last = agent.tools_used.last().unwrap();
                    format!("  → {}", last)
                };

                // Agent box
                lines.push(Line::from(vec![
                    Span::styled(format!("              {}", connector), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(" {} ", icon),
                        Style::default().fg(color),
                    ),
                    Span::styled(
                        format!("{:<20}", agent.agent_type),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" [{}]", status_label),
                        Style::default().fg(color),
                    ),
                    Span::styled(
                        format!("  calls: {}", agent.tools_used.len()),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(tool_summary, Style::default().fg(Color::Yellow)),
                ]));
            }
        }

        // Show parallel execution indicator
        if running_subs.len() > 1 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  ⚡ ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{} agents running in parallel", running_subs.len()),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
        }
    }

    let topology = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Agent Topology ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(topology, chunks[1]);

    // Stats panel
    let running = app.running_agents();
    let completed = app.completed_agents();
    let stats_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Agents    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} total", app.agents.len()), Style::default().fg(Color::White)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("● {} running", running), Style::default().fg(Color::Green)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("✓ {} done", completed), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("  Events    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.total_events), Style::default().fg(Color::Yellow)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Tool calls  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.tool_uses), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
    ];

    let stats = Paragraph::new(Text::from(stats_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Stats ")
                .border_style(Style::default().fg(Color::DarkGray)),
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
        // Refresh data every render cycle
        app.refresh();

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Tab bar
                    Constraint::Min(1),   // Content
                    Constraint::Length(1), // Status line
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
                        .title_style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .select(tab_index)
                .style(Style::default().fg(Color::DarkGray))
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
                Tab::Claude => render_claude(frame, chunks[1], &app),
                Tab::Observability => render_observability(frame, chunks[1], &app),
            }

            // Status line
            let status = Line::from(vec![
                Span::styled(
                    format!(
                        " Agents: {}  Events: {}  Tools: {} ",
                        app.agents.len(),
                        app.total_events,
                        app.tool_uses,
                    ),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if app.tab == Tab::Claude {
                        "Type to chat │ Enter: send │ Esc: back │ q: quit"
                    } else {
                        "1-3: switch tabs │ q/Esc: quit │ Refreshes every 2s"
                    },
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            frame.render_widget(Paragraph::new(status), chunks[2]);
        })?;

        // Handle input
        let timeout = if app.tab == Tab::Claude && !app.claude_loading {
            Duration::from_millis(100) // Faster polling for typing
        } else {
            Duration::from_secs(2) // Normal refresh
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Claude tab has special input handling
                if app.tab == Tab::Claude && !app.claude_loading {
                    match key.code {
                        KeyCode::Esc => {
                            app.tab = Tab::Overview;
                        }
                        KeyCode::Enter => {
                            if !app.claude_input.is_empty() {
                                let question = app.claude_input.clone();
                                app.claude_input.clear();
                                app.claude_loading = true;

                                // We need to drop raw mode briefly to run claude
                                disable_raw_mode()?;
                                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                                let answer = ask_claude_sync(&app.project_dir, &question);

                                execute!(io::stdout(), EnterAlternateScreen)?;
                                enable_raw_mode()?;

                                app.claude_history.push((question, answer));
                                app.claude_loading = false;
                            }
                        }
                        KeyCode::Backspace => {
                            app.claude_input.pop();
                        }
                        KeyCode::Char(c) => {
                            // Don't capture tab-switch keys while typing
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // pass through
                            } else {
                                app.claude_input.push(c);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Normal key handling
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
