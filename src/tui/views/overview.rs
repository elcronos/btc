use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Row, Table};

use crate::types::AgentSnapshot;

pub fn render_overview(
    frame: &mut Frame,
    area: Rect,
    agents: &[AgentSnapshot],
    selected: Option<usize>,
) {
    let header = Row::new(vec!["#", "ID", "Status", "Task", "Cost", "Duration"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let task_display = if agent.task.len() > 30 {
                format!("{}...", &agent.task[..27])
            } else {
                agent.task.clone()
            };

            let style = if selected == Some(i) {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                format!("{}", i + 1),
                agent.id.to_string(),
                agent.status.label().to_string(),
                task_display,
                format!("${:.4}", agent.cost),
                format!("{:.1}s", agent.duration_secs),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Agents"));

    frame.render_widget(table, area);
}
