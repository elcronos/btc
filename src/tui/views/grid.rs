use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::types::AgentSnapshot;

use crate::tui::widgets::status_badge::status_span;

pub fn render_grid(frame: &mut Frame, area: Rect, agents: &[AgentSnapshot]) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let cells = [top[0], top[1], bottom[0], bottom[1]];

    for (i, cell) in cells.iter().enumerate() {
        if let Some(agent) = agents.get(i) {
            render_agent_cell(frame, *cell, agent);
        }
    }
}

fn render_agent_cell(frame: &mut Frame, area: Rect, agent: &AgentSnapshot) {
    let badge = status_span(agent.status);
    let title_line = Line::from(vec![
        Span::raw(format!("{} ", agent.id)),
        badge,
    ]);

    let task_display = if agent.task.len() > 40 {
        format!("{}...", &agent.task[..37])
    } else {
        agent.task.clone()
    };

    let output_line = agent
        .last_output
        .as_deref()
        .unwrap_or("(no output)")
        .lines()
        .last()
        .unwrap_or("(no output)");

    let text = Text::from(vec![
        title_line,
        Line::from(format!("Task: {}", task_display)),
        Line::from(format!("Out:  {}", output_line)),
        Line::from(format!("Cost: ${:.4}", agent.cost)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Agent {}", agent.id));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
