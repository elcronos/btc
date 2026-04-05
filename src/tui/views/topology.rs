use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

pub fn render_topology(frame: &mut Frame, area: Rect, completed: u32, total: u32) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Progress gauge
    let ratio = if total > 0 {
        (completed as f64) / (total as f64)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("DAG Progress"),
        )
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(ratio)
        .label(format!("{}/{} nodes", completed, total));

    frame.render_widget(gauge, chunks[0]);

    // Placeholder node list
    let info = Paragraph::new("Node topology details will appear here.")
        .block(Block::default().borders(Borders::ALL).title("Nodes"));

    frame.render_widget(info, chunks[1]);
}
