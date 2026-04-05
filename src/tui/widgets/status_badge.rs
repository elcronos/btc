use ratatui::prelude::*;

use crate::types::AgentStatus;

pub fn status_style(status: AgentStatus) -> Style {
    let color = match status {
        AgentStatus::Spawning => Color::Magenta,
        AgentStatus::Running => Color::Green,
        AgentStatus::Idle => Color::Yellow,
        AgentStatus::Error => Color::Red,
        AgentStatus::Complete => Color::Blue,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub fn status_span(status: AgentStatus) -> Span<'static> {
    Span::styled(status.label().to_string(), status_style(status))
}
