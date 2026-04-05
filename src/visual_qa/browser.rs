use std::path::Path;

use crate::error::{BtcError, BtcResult};

/// Placeholder for headless browser integration.
/// Actual Chrome DevTools Protocol integration is deferred to implementation.
pub struct HeadlessBrowser {
    viewport: (u32, u32),
}

impl HeadlessBrowser {
    /// Attempt to create a new headless browser instance.
    /// Returns `BtcError::BrowserUnavailable` if Chrome/Chromium is not found.
    pub fn new(viewport: (u32, u32)) -> BtcResult<Self> {
        if !Self::is_available() {
            return Err(BtcError::BrowserUnavailable(
                "Chrome or Chromium not found on this system".into(),
            ));
        }
        Ok(Self { viewport })
    }

    /// Check whether a supported browser binary exists on the system.
    pub fn is_available() -> bool {
        // macOS Chrome.app
        if Path::new("/Applications/Google Chrome.app").exists() {
            return true;
        }
        // Check PATH for chromium / google-chrome
        for bin in &["chromium", "chromium-browser", "google-chrome"] {
            if std::process::Command::new("which")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return true;
            }
        }
        false
    }

    /// Capture a screenshot of the given URL and write it to `output_path`.
    /// Currently a placeholder — returns `BrowserUnavailable`.
    pub fn screenshot(&self, _url: &str, _output_path: &Path) -> BtcResult<()> {
        Err(BtcError::BrowserUnavailable(
            "Screenshot capture not yet implemented".into(),
        ))
    }

    /// Return the configured viewport dimensions.
    pub fn viewport(&self) -> (u32, u32) {
        self.viewport
    }

    /// Close the browser and release resources.
    pub fn close(self) -> BtcResult<()> {
        // Nothing to release yet.
        Ok(())
    }
}
