use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::types::AgentSnapshot;

pub fn render_cost(frame: &mut Frame, area: Rect, total_cost: f64, agents: &[AgentSnapshot]) {
    let mut lines = vec![Line::from(format!("Total cost: ${:.4}", total_cost))];

    for agent in agents {
        lines.push(Line::from(format!(
            "  {} — ${:.4}",
            agent.id, agent.cost
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Costs"));

    frame.render_widget(paragraph, area);
}
