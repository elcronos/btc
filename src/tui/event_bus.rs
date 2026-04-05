use tokio::sync::mpsc;
use tracing::warn;

use crate::types::{AgentSnapshot, MetricsSnapshot};

#[derive(Debug, Clone)]
pub enum TuiEvent {
    AgentUpdate(AgentSnapshot),
    MetricsUpdate(MetricsSnapshot),
    CheckpointScore { iteration: u32, score: f64 },
    DagProgress { completed: u32, total: u32 },
    Notification(String),
}

pub struct EventBus;

impl EventBus {
    pub fn new() -> (EventBusSender, EventBusReceiver) {
        let (tx, rx) = mpsc::channel(256);
        (EventBusSender { tx }, EventBusReceiver { rx })
    }
}

pub struct EventBusSender {
    tx: mpsc::Sender<TuiEvent>,
}

impl EventBusSender {
    pub fn try_send(&self, event: TuiEvent) {
        if let Err(e) = self.tx.try_send(event) {
            warn!("TUI event bus full, dropping event: {}", e);
        }
    }
}

pub struct EventBusReceiver {
    rx: mpsc::Receiver<TuiEvent>,
}

impl EventBusReceiver {
    pub fn try_recv(&mut self) -> Option<TuiEvent> {
        self.rx.try_recv().ok()
    }
}
