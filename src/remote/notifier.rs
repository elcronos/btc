use crate::error::BtcResult;

pub struct Notifier {
    channels: Vec<String>,
}

impl Notifier {
    pub fn new(channels: Vec<String>) -> Self {
        Self { channels }
    }

    /// Send a notification to all configured channels.
    ///
    /// Placeholder -- logs the notification for now.
    pub fn notify(&self, title: &str, body: &str) -> BtcResult<()> {
        for channel in &self.channels {
            tracing::info!(
                channel = %channel,
                title = %title,
                body = %body,
                "Sending notification"
            );
        }
        Ok(())
    }

    /// Notify that a task has completed.
    pub fn notify_completion(&self, summary: &str) -> BtcResult<()> {
        self.notify("Task Completed", summary)
    }

    /// Notify that an error occurred.
    pub fn notify_error(&self, error: &str) -> BtcResult<()> {
        self.notify("Error", error)
    }
}
