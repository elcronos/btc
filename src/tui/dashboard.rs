use std::io;
use std::path::Path;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};

use crate::error::BtcResult;
use crate::types::{AgentId, AgentSnapshot, AgentStatus, MetricsSnapshot};

use super::views::{render_agent_tree, render_focus, render_grid, render_overview};
use super::widgets::cost_tracker::render_cost;
use crate::observer::estimate_usage;

/// Scan system for running `claude` processes and return as agent snapshots.
fn scan_claude_processes() -> Vec<AgentSnapshot> {
    let output = std::process::Command::new("ps")
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
        if !line.contains("claude") || line.contains("grep") || line.contains("ps -eo") {
            continue;
        }
        // Skip the btc process itself
        if line.contains("btc") {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, char::is_whitespace).collect();
        if parts.len() < 3 {
            continue;
        }

        let pid = parts[0].trim();
        let etime = parts[1].trim();
        let cmd = parts[2].trim();

        // Parse elapsed time (format: [[dd-]hh:]mm:ss)
        let duration_secs = parse_etime(etime);

        // Extract task description from -p argument
        let task = extract_prompt(cmd)
            .unwrap_or_else(|| truncate_str(cmd, 60).to_string());

        agents.push(AgentSnapshot {
            id: AgentId(format!("pid-{}", pid)),
            status: AgentStatus::Running,
            task,
            cost: 0.0,
            duration_secs,
            event_count: 0,
            last_output: Some(truncate_str(cmd, 120).to_string()),
        });
    }

    agents
}

fn parse_etime(etime: &str) -> f64 {
    // Formats: "ss", "mm:ss", "hh:mm:ss", "dd-hh:mm:ss"
    let parts: Vec<&str> = etime.split(':').collect();
    match parts.len() {
        1 => parts[0].parse::<f64>().unwrap_or(0.0),
        2 => {
            let mins: f64 = parts[0].parse().unwrap_or(0.0);
            let secs: f64 = parts[1].parse().unwrap_or(0.0);
            mins * 60.0 + secs
        }
        3 => {
            let first = parts[0];
            let (days, hours) = if first.contains('-') {
                let dp: Vec<&str> = first.split('-').collect();
                (
                    dp[0].parse::<f64>().unwrap_or(0.0),
                    dp.get(1).and_then(|h| h.parse::<f64>().ok()).unwrap_or(0.0),
                )
            } else {
                (0.0, first.parse::<f64>().unwrap_or(0.0))
            };
            let mins: f64 = parts[1].parse().unwrap_or(0.0);
            let secs: f64 = parts[2].parse().unwrap_or(0.0);
            days * 86400.0 + hours * 3600.0 + mins * 60.0 + secs
        }
        _ => 0.0,
    }
}

fn extract_prompt(cmd: &str) -> Option<String> {
    // Look for -p "..." or -p '...' in command
    let idx = cmd.find(" -p ")?;
    let after = &cmd[idx + 4..];
    let trimmed = after.trim_start();
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        let quote = trimmed.chars().next()?;
        let rest = &trimmed[1..];
        let end = rest.find(quote).unwrap_or(rest.len());
        Some(truncate_str(&rest[..end], 80).to_string())
    } else {
        // Unquoted - take until next flag
        let end = trimmed.find(" -").unwrap_or(trimmed.len());
        Some(truncate_str(&trimmed[..end], 80).to_string())
    }
}

fn truncate_str(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

/// Read latest plan state from .btc/
fn read_dag_state(project_dir: &Path) -> (u32, u32) {
    let plans_dir = project_dir.join(".btc").join("plans");
    let entries = match std::fs::read_dir(&plans_dir) {
        Ok(e) => e,
        Err(_) => return (0, 0),
    };

    let mut latest: Option<std::path::PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            latest = Some(path);
        }
    }

    if let Some(path) = latest {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(graph) = serde_json::from_str::<crate::dag::graph::TaskGraph>(&content) {
                let total = graph.node_count() as u32;
                let completed = graph.completed_count() as u32;
                return (completed, total);
            }
        }
    }

    (0, 0)
}

#[derive(PartialEq, Clone)]
enum Tab {
    Overview,
    Grid,
    Topology,
    Costs,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Grid => "Grid",
            Tab::Topology => "DAG",
            Tab::Costs => "Costs",
        }
    }

    fn all() -> Vec<Tab> {
        vec![Tab::Overview, Tab::Grid, Tab::Topology, Tab::Costs]
    }
}

pub fn run_dashboard(project_dir: &Path) -> BtcResult<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_tab = Tab::Overview;
    let mut selected_agent: usize = 0;
    let mut agents: Vec<AgentSnapshot> = Vec::new();
    let tabs = Tab::all();
    let mut focus_agent: Option<AgentId> = None;

    loop {
        // Refresh agent data
        agents = scan_claude_processes();
        let (dag_completed, dag_total) = read_dag_state(project_dir);

        let total_cost: f64 = agents.iter().map(|a| a.cost).sum();
        let (total_events, _tool_uses) = estimate_usage(project_dir);
        let metrics = MetricsSnapshot {
            total_agents: agents.len(),
            running_agents: agents.iter().filter(|a| a.status == AgentStatus::Running).count(),
            total_cost,
            elapsed_secs: agents.iter().map(|a| a.duration_secs).fold(0.0_f64, f64::max),
            checkpoints_created: 0,
            best_score: None,
            timestamp: chrono::Utc::now(),
        };

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header + tabs
                    Constraint::Min(1),   // Main content
                    Constraint::Length(3), // Footer
                ])
                .split(frame.area());

            // Header with tabs
            let tab_titles: Vec<Line> = tabs
                .iter()
                .map(|t| {
                    if *t == current_tab {
                        Line::from(t.label()).style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Line::from(t.label())
                    }
                })
                .collect();

            let tab_index = tabs.iter().position(|t| *t == current_tab).unwrap_or(0);
            let tabs_widget = Tabs::new(tab_titles)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" ⚡ BTC Dashboard ")
                        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                )
                .select(tab_index)
                .style(Style::default().fg(Color::DarkGray))
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_widget(tabs_widget, chunks[0]);

            // Main content area
            if let Some(ref fid) = focus_agent {
                if let Some(agent) = agents.iter().find(|a| a.id == *fid) {
                    render_focus(frame, chunks[1], agent);
                } else {
                    focus_agent = None;
                    render_empty(frame, chunks[1], &agents, &metrics);
                }
            } else {
                match current_tab {
                    Tab::Overview => {
                        render_overview(
                            frame,
                            chunks[1],
                            &agents,
                            Some(selected_agent),
                        );
                    }
                    Tab::Grid => {
                        render_grid(frame, chunks[1], &agents);
                    }
                    Tab::Topology => {
                        render_agent_tree(frame, chunks[1], project_dir, dag_completed, dag_total);
                    }
                    Tab::Costs => {
                        render_cost(frame, chunks[1], total_cost, &agents);
                    }
                }
            }

            // Footer
            let footer_text = if focus_agent.is_some() {
                " Esc: back │ q: quit │ Refreshing every 2s "
            } else {
                " 1-4: tabs │ Tab: select │ Enter: focus │ q: quit │ Refreshing every 2s "
            };
            let status_line = format!(
                " Agents: {}  Running: {}  Events: {}  Cost: ${:.4}  Uptime: {:.0}s ",
                metrics.total_agents,
                metrics.running_agents,
                total_events,
                metrics.total_cost,
                metrics.elapsed_secs,
            );

            let footer = Paragraph::new(Line::from(vec![
                Span::styled(footer_text, Style::default().fg(Color::DarkGray)),
                Span::styled(status_line, Style::default().fg(Color::Cyan)),
            ]))
            .block(Block::default().borders(Borders::ALL));

            frame.render_widget(footer, chunks[2]);
        })?;

        // Handle input with 2s timeout (refresh rate)
        if event::poll(Duration::from_secs(2))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => {
                        if focus_agent.is_some() {
                            focus_agent = None;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Char('1') => { current_tab = Tab::Overview; focus_agent = None; }
                    KeyCode::Char('2') => { current_tab = Tab::Grid; focus_agent = None; }
                    KeyCode::Char('3') => { current_tab = Tab::Topology; focus_agent = None; }
                    KeyCode::Char('4') => { current_tab = Tab::Costs; focus_agent = None; }
                    KeyCode::Tab => {
                        if !agents.is_empty() {
                            selected_agent = (selected_agent + 1) % agents.len();
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(agent) = agents.get(selected_agent) {
                            focus_agent = Some(agent.id.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn render_empty(frame: &mut Frame, area: Rect, agents: &[AgentSnapshot], metrics: &MetricsSnapshot) {
    let text = if agents.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No Claude agents detected.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Run 'btc run' in another terminal to start agents.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  The dashboard refreshes every 2 seconds.",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(format!("  {} agents running", metrics.running_agents)),
        ]
    };

    let paragraph = Paragraph::new(Text::from(text))
        .block(Block::default().borders(Borders::ALL).title("Dashboard"))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
