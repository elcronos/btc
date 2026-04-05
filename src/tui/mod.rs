mod app;
pub mod dashboard;
mod event_bus;
mod input;
pub mod views;
pub mod widgets;

pub use app::{App, ViewMode};
pub use dashboard::run_dashboard;
pub use event_bus::{EventBus, EventBusReceiver, EventBusSender, TuiEvent};
