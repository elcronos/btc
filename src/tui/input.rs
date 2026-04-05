use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};

use crate::error::BtcResult;

pub fn handle_input(timeout: Duration) -> BtcResult<Option<KeyEvent>> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}
