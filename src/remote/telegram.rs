use crate::error::BtcResult;

use super::router::CommandRouter;

/// Adapter for handling Telegram bot messages.
///
/// Placeholder -- actual teloxide bot integration deferred.
pub struct TelegramAdapter {
    _router: CommandRouter,
}

impl TelegramAdapter {
    pub fn new(router: CommandRouter) -> Self {
        Self { _router: router }
    }

    /// Handle an incoming Telegram message and return a response string.
    pub fn handle_message(&self, _chat_id: i64, text: &str) -> BtcResult<String> {
        let cmd = CommandRouter::parse(text)?;
        let response = format!("{cmd:?}");
        Ok(response)
    }
}
