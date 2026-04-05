use std::path::PathBuf;

use crate::error::BtcResult;

use super::browser::HeadlessBrowser;

/// Captures screenshots for a set of routes using a headless browser.
pub struct ScreenshotCapture {
    browser: HeadlessBrowser,
    output_dir: PathBuf,
}

impl ScreenshotCapture {
    /// Create a new capture instance.
    /// Returns an error if the browser is unavailable.
    pub fn new(viewport: (u32, u32), output_dir: PathBuf) -> BtcResult<Self> {
        let browser = HeadlessBrowser::new(viewport)?;
        Ok(Self {
            browser,
            output_dir,
        })
    }

    /// Capture screenshots for each route and return the list of output paths.
    pub fn capture_routes(&self, routes: &[String]) -> BtcResult<Vec<PathBuf>> {
        std::fs::create_dir_all(&self.output_dir)?;

        let mut paths = Vec::with_capacity(routes.len());
        for route in routes {
            let filename = Self::route_to_filename(route);
            let output_path = self.output_dir.join(&filename);
            match self.browser.screenshot(route, &output_path) {
                Ok(()) => paths.push(output_path),
                Err(e) => {
                    tracing::warn!(route = %route, error = %e, "Failed to capture screenshot");
                    return Err(e);
                }
            }
        }

        Ok(paths)
    }

    /// Convert a route URL to a safe filename.
    fn route_to_filename(route: &str) -> String {
        let safe: String = route
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
            .collect();
        format!("{}.png", safe)
    }
}
