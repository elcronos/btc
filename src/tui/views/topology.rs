use std::path::Path;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::observer::{read_agent_topology, estimate_usage, AgentTrackStatus};

pub fn render_topology(frame: &mut Frame, area: Rect, completed: u32, total: u32) {
    // Legacy overload — no project_dir, just show DAG progress
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let ratio = if total > 0 {
        (completed as f64) / (total as f64)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("DAG Progress"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(ratio)
        .label(format!("{}/{} nodes", completed, total));

    frame.render_widget(gauge, chunks[0]);

    let info = Paragraph::new("No project directory available.")
        .block(Block::default().borders(Borders::ALL).title("Agents"));

    frame.render_widget(info, chunks[1]);
}

pub fn render_agent_tree(frame: &mut Frame, area: Rect, project_dir: &Path, completed: u32, total: u32) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // DAG progress gauge at top
    let ratio = if total > 0 {
        (completed as f64) / (total as f64)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("DAG Progress"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(ratio)
        .label(format!("{}/{} nodes", completed, total));

    frame.render_widget(gauge, chunks[0]);

    // Agent tree
    let agents = read_agent_topology(project_dir);
    let (total_events, _tool_uses) = estimate_usage(project_dir);

    let mut lines: Vec<Line> = Vec::new();

    if agents.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No agent events found in .btc/agent-events.jsonl",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Run 'btc run' to start agents. Events will appear here.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Separate main session agent from subagents
        let main_agents: Vec<_> = agents.iter()
            .filter(|a| a.agent_type == "main")
            .collect();
        let sub_agents: Vec<_> = agents.iter()
            .filter(|a| a.agent_type != "main")
            .collect();

        lines.push(Line::from(""));

        // Render main session entries
        for main in &main_agents {
            let (status_label, status_color) = match main.status {
                AgentTrackStatus::Running => ("[RUN]", Color::Green),
                AgentTrackStatus::Completed => ("[OK] ", Color::Blue),
            };
            let id_short = if main.id.len() > 12 { &main.id[..12] } else { &main.id };
            lines.push(Line::from(vec![
                Span::styled("  Main Session ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!("({})", id_short), Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(status_label, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            ]));
        }

        if main_agents.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Main Session ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("[RUN]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
        }

        // Render subagents as tree children
        let sub_count = sub_agents.len();
        for (i, agent) in sub_agents.iter().enumerate() {
            let is_last = i == sub_count - 1;
            let tree_char = if is_last { "\u{2514}\u{2500}\u{2500}" } else { "\u{251C}\u{2500}\u{2500}" };

            let (status_label, status_color) = match agent.status {
                AgentTrackStatus::Running => ("[RUN]", Color::Green),
                AgentTrackStatus::Completed => ("[OK] ", Color::Blue),
            };

            let last_tool = agent.tools_used.last().cloned().unwrap_or_default();
            let tool_hint = if !last_tool.is_empty() {
                format!(" \u{2014} {}", last_tool)
            } else {
                String::new()
            };

            let id_short = if agent.id.len() > 12 { &agent.id[..12] } else { &agent.id };

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", tree_char), Style::default().fg(Color::DarkGray)),
                Span::styled(agent.agent_type.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("({})", id_short), Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(status_label, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(tool_hint, Style::default().fg(Color::DarkGray)),
            ]));
        }

        if total_events > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  Total events logged: {}", total_events),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Agent Topology"));

    frame.render_widget(paragraph, chunks[1]);
}
