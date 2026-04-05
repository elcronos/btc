use crate::error::BtcResult;

/// Show a desktop notification. Gracefully handles failure.
pub fn notify_desktop(title: &str, body: &str) -> BtcResult<()> {
    if let Err(e) = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show()
    {
        tracing::warn!("Desktop notification failed: {}", e);
    }
    Ok(())
}

/// Send a completion notification.
pub fn notify_completion(project: &str, summary: &str) -> BtcResult<()> {
    notify_desktop(
        &format!("BTC: {} complete", project),
        summary,
    )
}

/// Send an error notification.
pub fn notify_error(project: &str, error: &str) -> BtcResult<()> {
    notify_desktop(
        &format!("BTC: {} error", project),
        error,
    )
}
