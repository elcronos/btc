use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::types::AgentSnapshot;

use crate::tui::widgets::status_badge::status_span;

pub fn render_focus(frame: &mut Frame, area: Rect, agent: &AgentSnapshot) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Header
    let header_line = Line::from(vec![
        Span::styled(
            format!(" {} ", agent.id),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        status_span(agent.status),
        Span::raw(format!(
            "  Cost: ${:.4}  Duration: {:.1}s",
            agent.cost, agent.duration_secs
        )),
    ]);

    let header = Paragraph::new(header_line)
        .block(Block::default().borders(Borders::ALL).title("Focus"));

    frame.render_widget(header, chunks[0]);

    // Body — last output
    let output_text = agent
        .last_output
        .as_deref()
        .unwrap_or("(no output yet)");

    let body = Paragraph::new(output_text)
        .block(Block::default().borders(Borders::ALL).title("Output"))
        .wrap(Wrap { trim: false });

    frame.render_widget(body, chunks[1]);
}
