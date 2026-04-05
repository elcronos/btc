use std::path::Path;

use crate::error::BtcResult;

use super::critique::AssetCritic;
use super::generator::AssetGenerator;
use super::staging::StagingWorkspace;
use super::types::{AssetPipelineConfig, AssetResult, AssetSpec, AssetVerdict};

/// Orchestrates the generate-critique loop for asset production.
pub struct AssetPipeline<G: AssetGenerator> {
    generator: G,
    critic: AssetCritic,
    config: AssetPipelineConfig,
}

impl<G: AssetGenerator> AssetPipeline<G> {
    pub fn new(generator: G, critic: AssetCritic, config: AssetPipelineConfig) -> Self {
        Self {
            generator,
            critic,
            config,
        }
    }

    /// Run the generate-critique loop for a single asset spec.
    ///
    /// 1. Create staging workspace
    /// 2. Generate asset
    /// 3. Critique asset
    /// 4. If Accept and score >= threshold: promote and return
    /// 5. If Refine: regenerate with feedback (up to max_iterations)
    /// 6. After max_iterations: promote the best-scoring asset
    /// 7. Cleanup staging
    pub async fn process(
        &self,
        spec: &AssetSpec,
        project_dest: &Path,
    ) -> BtcResult<AssetResult> {
        let batch_id = format!("asset-{}", spec.name);
        let workspace = StagingWorkspace::new(&batch_id, &self.config.staging_dir)?;

        let mut best_score: f64 = 0.0;
        let mut best_path = None;
        let mut best_iteration: u32 = 0;
        let mut feedback: Option<String> = None;

        for iteration in 1..=self.config.max_iterations {
            let generated = self
                .generator
                .generate(spec, feedback.as_deref(), workspace.root())
                .await?;

            let verdict = self.critic.critique(&generated, spec)?;

            match verdict {
                AssetVerdict::Accept { score } => {
                    if score > best_score {
                        best_score = score;
                        best_path = Some(generated.clone());
                        best_iteration = iteration;
                    }
                    if score >= self.config.score_threshold {
                        StagingWorkspace::promote_to_project(&generated, project_dest)?;
                        workspace.cleanup()?;
                        return Ok(AssetResult {
                            path: project_dest.to_path_buf(),
                            score,
                            iteration,
                            format: spec.format,
                        });
                    }
                    // Score below threshold — continue iterating
                    feedback = Some(format!(
                        "Score {:.1} is below threshold {:.1}. Improve quality.",
                        score, self.config.score_threshold
                    ));
                }
                AssetVerdict::Refine {
                    score,
                    feedback: fb,
                } => {
                    if score > best_score {
                        best_score = score;
                        best_path = Some(generated.clone());
                        best_iteration = iteration;
                    }
                    feedback = Some(fb);
                }
                AssetVerdict::Reject { reason } => {
                    feedback = Some(format!("Rejected: {}", reason));
                }
            }
        }

        // Max iterations reached — promote the best asset we have
        if let Some(best) = best_path {
            StagingWorkspace::promote_to_project(&best, project_dest)?;
        }
        workspace.cleanup()?;

        Ok(AssetResult {
            path: project_dest.to_path_buf(),
            score: best_score,
            iteration: best_iteration,
            format: spec.format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_pipeline::generator::SvgGenerator;
    use crate::asset_pipeline::types::AssetFormat;
    use tempfile::TempDir;

    #[tokio::test]
    async fn pipeline_exits_with_best_score() {
        let tmp = TempDir::new().unwrap();
        let staging_dir = tmp.path().join("staging");
        std::fs::create_dir_all(&staging_dir).unwrap();

        let config = AssetPipelineConfig {
            max_iterations: 3,
            score_threshold: 95.0, // Unreachable — forces max iterations
            staging_dir,
        };

        let generator = SvgGenerator::new();
        // Fallback critic returns 75.0 each time
        let critic = AssetCritic::new(false);
        let pipeline = AssetPipeline::new(generator, critic, config);

        let spec = AssetSpec {
            name: "logo".into(),
            description: "A test logo".into(),
            format: AssetFormat::Svg,
            dimensions: Some((100, 100)),
        };

        let dest = tmp.path().join("output/logo.svg");
        let result = pipeline.process(&spec, &dest).await.unwrap();

        assert_eq!(result.score, 75.0);
        assert!(result.iteration >= 1);
        assert!(dest.exists());
    }

    #[tokio::test]
    async fn pipeline_accepts_above_threshold() {
        let tmp = TempDir::new().unwrap();
        let staging_dir = tmp.path().join("staging");
        std::fs::create_dir_all(&staging_dir).unwrap();

        let config = AssetPipelineConfig {
            max_iterations: 5,
            score_threshold: 70.0, // Fallback returns 75 — should accept on first iteration
            staging_dir,
        };

        let generator = SvgGenerator::new();
        let critic = AssetCritic::new(false);
        let pipeline = AssetPipeline::new(generator, critic, config);

        let spec = AssetSpec {
            name: "icon".into(),
            description: "A test icon".into(),
            format: AssetFormat::Svg,
            dimensions: None,
        };

        let dest = tmp.path().join("output/icon.svg");
        let result = pipeline.process(&spec, &dest).await.unwrap();

        assert!(result.score >= 70.0);
        assert_eq!(result.iteration, 1);
        assert!(dest.exists());
    }
}
