use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent};

use crate::types::{AgentId, AgentSnapshot, MetricsSnapshot};

use super::event_bus::TuiEvent;

const MAX_NOTIFICATIONS: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Grid,
    Overview,
    Focus(AgentId),
    Topology,
}

pub struct App {
    pub view_mode: ViewMode,
    pub agents: Vec<AgentSnapshot>,
    pub metrics: MetricsSnapshot,
    pub dag_completed: u32,
    pub dag_total: u32,
    pub notifications: VecDeque<String>,
    pub should_quit: bool,
    pub help_visible: bool,
    pub selected_index: usize,
    manual_view: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Grid,
            agents: Vec::new(),
            metrics: MetricsSnapshot::default(),
            dag_completed: 0,
            dag_total: 0,
            notifications: VecDeque::new(),
            should_quit: false,
            help_visible: false,
            selected_index: 0,
            manual_view: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.help_visible = !self.help_visible,
            KeyCode::Char('g') => {
                self.view_mode = ViewMode::Grid;
                self.manual_view = true;
            }
            KeyCode::Char('o') => {
                self.view_mode = ViewMode::Overview;
                self.manual_view = true;
            }
            KeyCode::Char('f') => {
                if let Some(agent) = self.agents.get(self.selected_index) {
                    self.view_mode = ViewMode::Focus(agent.id.clone());
                    self.manual_view = true;
                }
            }
            KeyCode::Char('t') => {
                self.view_mode = ViewMode::Topology;
                self.manual_view = true;
            }
            KeyCode::Tab => {
                if !self.agents.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.agents.len();
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let idx = (c as usize) - ('1' as usize);
                if idx < self.agents.len() {
                    self.selected_index = idx;
                    self.view_mode = ViewMode::Focus(self.agents[idx].id.clone());
                    self.manual_view = true;
                }
            }
            _ => {}
        }
    }

    pub fn update_from_event(&mut self, event: TuiEvent) {
        match event {
            TuiEvent::AgentUpdate(snapshot) => {
                if let Some(existing) = self.agents.iter_mut().find(|a| a.id == snapshot.id) {
                    *existing = snapshot;
                } else {
                    self.agents.push(snapshot);
                }
                self.auto_view_mode();
            }
            TuiEvent::MetricsUpdate(metrics) => {
                self.metrics = metrics;
            }
            TuiEvent::CheckpointScore { iteration, score } => {
                self.push_notification(format!(
                    "Checkpoint #{}: score {:.1}",
                    iteration, score
                ));
            }
            TuiEvent::DagProgress { completed, total } => {
                self.dag_completed = completed;
                self.dag_total = total;
            }
            TuiEvent::Notification(msg) => {
                self.push_notification(msg);
            }
        }
    }

    pub fn auto_view_mode(&mut self) {
        if self.manual_view {
            return;
        }
        if self.agents.len() <= 4 {
            self.view_mode = ViewMode::Grid;
        } else {
            self.view_mode = ViewMode::Overview;
        }
    }

    pub fn selected_agent(&self) -> Option<&AgentSnapshot> {
        self.agents.get(self.selected_index)
    }

    fn push_notification(&mut self, msg: String) {
        if self.notifications.len() >= MAX_NOTIFICATIONS {
            self.notifications.pop_front();
        }
        self.notifications.push_back(msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn make_agent(id: &str) -> AgentSnapshot {
        AgentSnapshot {
            id: AgentId(id.to_string()),
            status: crate::types::AgentStatus::Running,
            task: "test task".to_string(),
            cost: 0.01,
            duration_secs: 1.0,
            event_count: 0,
            last_output: None,
        }
    }

    #[test]
    fn handle_key_quit() {
        let mut app = App::new();
        assert!(!app.should_quit);
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn handle_key_view_transitions() {
        let mut app = App::new();
        app.agents.push(make_agent("a1"));

        app.handle_key(key(KeyCode::Char('o')));
        assert_eq!(app.view_mode, ViewMode::Overview);

        app.handle_key(key(KeyCode::Char('g')));
        assert_eq!(app.view_mode, ViewMode::Grid);

        app.handle_key(key(KeyCode::Char('t')));
        assert_eq!(app.view_mode, ViewMode::Topology);

        app.handle_key(key(KeyCode::Char('f')));
        assert_eq!(app.view_mode, ViewMode::Focus(AgentId("a1".to_string())));
    }

    #[test]
    fn handle_key_digit_focus() {
        let mut app = App::new();
        app.agents.push(make_agent("a1"));
        app.agents.push(make_agent("a2"));

        app.handle_key(key(KeyCode::Char('2')));
        assert_eq!(app.selected_index, 1);
        assert_eq!(app.view_mode, ViewMode::Focus(AgentId("a2".to_string())));
    }

    #[test]
    fn handle_key_tab_cycles() {
        let mut app = App::new();
        app.agents.push(make_agent("a1"));
        app.agents.push(make_agent("a2"));
        app.agents.push(make_agent("a3"));

        assert_eq!(app.selected_index, 0);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.selected_index, 1);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.selected_index, 2);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn handle_key_help_toggle() {
        let mut app = App::new();
        assert!(!app.help_visible);
        app.handle_key(key(KeyCode::Char('?')));
        assert!(app.help_visible);
        app.handle_key(key(KeyCode::Char('?')));
        assert!(!app.help_visible);
    }

    #[test]
    fn auto_view_mode_switches_on_agent_count() {
        let mut app = App::new();
        assert_eq!(app.view_mode, ViewMode::Grid);

        // Add 5 agents — should switch to Overview
        for i in 0..5 {
            app.agents.push(make_agent(&format!("a{}", i)));
        }
        app.auto_view_mode();
        assert_eq!(app.view_mode, ViewMode::Overview);

        // Remove agents back to 4 — should switch to Grid
        app.agents.truncate(4);
        app.auto_view_mode();
        assert_eq!(app.view_mode, ViewMode::Grid);
    }

    #[test]
    fn auto_view_mode_respects_manual() {
        let mut app = App::new();
        for i in 0..5 {
            app.agents.push(make_agent(&format!("a{}", i)));
        }

        // Manually set to Topology
        app.handle_key(key(KeyCode::Char('t')));
        assert_eq!(app.view_mode, ViewMode::Topology);

        // auto_view_mode should not override manual selection
        app.auto_view_mode();
        assert_eq!(app.view_mode, ViewMode::Topology);
    }

    #[test]
    fn update_from_event_agent() {
        let mut app = App::new();
        let agent = make_agent("a1");
        app.update_from_event(TuiEvent::AgentUpdate(agent));
        assert_eq!(app.agents.len(), 1);
        assert_eq!(app.agents[0].id, AgentId("a1".to_string()));
    }

    #[test]
    fn update_from_event_dag_progress() {
        let mut app = App::new();
        app.update_from_event(TuiEvent::DagProgress {
            completed: 3,
            total: 10,
        });
        assert_eq!(app.dag_completed, 3);
        assert_eq!(app.dag_total, 10);
    }

    #[test]
    fn notifications_cap_at_max() {
        let mut app = App::new();
        for i in 0..15 {
            app.update_from_event(TuiEvent::Notification(format!("msg {}", i)));
        }
        assert_eq!(app.notifications.len(), MAX_NOTIFICATIONS);
        assert_eq!(app.notifications.front().unwrap(), "msg 5");
    }
}
