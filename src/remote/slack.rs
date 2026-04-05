use crate::error::BtcResult;

use super::router::CommandRouter;

/// Adapter for handling Slack messages.
///
/// Placeholder -- actual slack-morphism integration deferred.
pub struct SlackAdapter {
    _router: CommandRouter,
}

impl SlackAdapter {
    pub fn new(router: CommandRouter) -> Self {
        Self { _router: router }
    }

    /// Handle an incoming Slack message and return a response string.
    pub fn handle_message(&self, _user_id: &str, text: &str) -> BtcResult<String> {
        let cmd = CommandRouter::parse(text)?;
        let response = format!("{cmd:?}");
        Ok(response)
    }
}
