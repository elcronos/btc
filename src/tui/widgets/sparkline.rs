use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Sparkline as RatatuiSparkline};

pub fn render_sparkline(frame: &mut Frame, area: Rect, data: &[f64]) {
    let int_data: Vec<u64> = data.iter().map(|v| (*v * 100.0) as u64).collect();

    let sparkline = RatatuiSparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Sparkline"))
        .data(&int_data)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, area);
}
