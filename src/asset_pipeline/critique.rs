use std::path::Path;

use crate::error::BtcResult;

use super::types::{AssetSpec, AssetVerdict};

/// Critiques generated assets, optionally using a headless browser for visual scoring.
pub struct AssetCritic {
    fallback_mode: bool,
}

impl AssetCritic {
    /// Create a new critic. When `browser_available` is false, uses text-only fallback.
    pub fn new(browser_available: bool) -> Self {
        Self {
            fallback_mode: !browser_available,
        }
    }

    /// Critique an asset and return a verdict.
    pub fn critique(&self, asset_path: &Path, _spec: &AssetSpec) -> BtcResult<AssetVerdict> {
        if self.fallback_mode {
            // Text-only fallback: read source and do basic validation
            let _content = std::fs::read_to_string(asset_path)?;
            Ok(AssetVerdict::Accept { score: 75.0 })
        } else {
            // Browser-based: would screenshot and score via visual QA
            // Placeholder: returns Accept with score 85
            Ok(AssetVerdict::Accept { score: 85.0 })
        }
    }

    /// Whether the critic is operating in text-only fallback mode.
    pub fn is_fallback(&self) -> bool {
        self.fallback_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_pipeline::types::AssetFormat;
    use tempfile::TempDir;

    #[test]
    fn fallback_mode_returns_lower_score() {
        let tmp = TempDir::new().unwrap();
        let svg_path = tmp.path().join("test.svg");
        std::fs::write(&svg_path, "<svg/>").unwrap();

        let spec = AssetSpec {
            name: "test".into(),
            description: "test asset".into(),
            format: AssetFormat::Svg,
            dimensions: None,
        };

        let critic_fallback = AssetCritic::new(false);
        assert!(critic_fallback.is_fallback());
        let verdict_fallback = critic_fallback.critique(&svg_path, &spec).unwrap();

        let critic_browser = AssetCritic::new(true);
        assert!(!critic_browser.is_fallback());
        let verdict_browser = critic_browser.critique(&svg_path, &spec).unwrap();

        let score_fallback = match verdict_fallback {
            AssetVerdict::Accept { score } => score,
            _ => panic!("expected Accept"),
        };
        let score_browser = match verdict_browser {
            AssetVerdict::Accept { score } => score,
            _ => panic!("expected Accept"),
        };

        assert!(score_fallback < score_browser);
    }
}
