use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};

pub fn render_progress(frame: &mut Frame, area: Rect, completed: u32, total: u32) {
    let ratio = if total > 0 {
        (completed as f64) / (total as f64)
    } else {
        0.0
    };

    let pct = (ratio * 100.0) as u16;

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Progress"),
        )
        .gauge_style(Style::default().fg(Color::Green))
        .percent(pct);

    frame.render_widget(gauge, area);
}
