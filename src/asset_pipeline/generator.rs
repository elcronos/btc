use std::path::{Path, PathBuf};

use crate::error::BtcResult;

use super::types::AssetSpec;

/// Object-safe trait for asset generators.
pub trait AssetGenerator: Send + Sync {
    fn name(&self) -> &str;

    fn generate(
        &self,
        spec: &AssetSpec,
        feedback: Option<&str>,
        output_dir: &Path,
    ) -> impl std::future::Future<Output = BtcResult<PathBuf>> + Send;
}

/// Simple SVG placeholder generator for testing.
pub struct SvgGenerator;

impl SvgGenerator {
    pub fn new() -> Self {
        Self
    }

    fn build_svg(spec: &AssetSpec, feedback: Option<&str>) -> String {
        let (width, height) = spec.dimensions.unwrap_or((200, 200));
        let fill = if feedback.is_some() {
            "#4a90d9"
        } else {
            "#6c757d"
        };
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" fill="{fill}" rx="8"/>
  <text x="50%" y="50%" text-anchor="middle" dominant-baseline="middle" fill="white" font-size="16" font-family="sans-serif">{name}</text>
</svg>"#,
            width = width,
            height = height,
            fill = fill,
            name = spec.name,
        )
    }
}

impl AssetGenerator for SvgGenerator {
    fn name(&self) -> &str {
        "svg-placeholder"
    }

    async fn generate(
        &self,
        spec: &AssetSpec,
        feedback: Option<&str>,
        output_dir: &Path,
    ) -> BtcResult<PathBuf> {
        let svg_content = Self::build_svg(spec, feedback);
        let file_name = format!("{}.svg", spec.name);
        let output_path = output_dir.join(&file_name);

        tokio::fs::write(&output_path, svg_content).await?;
        Ok(output_path)
    }
}
