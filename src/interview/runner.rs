use std::path::PathBuf;

use crate::error::BtcResult;

pub struct InterviewRunner {
    project_dir: PathBuf,
}

impl InterviewRunner {
    pub fn new(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }

    /// Run a placeholder interview that creates a spec file.
    ///
    /// TODO: Implement actual Socratic interview loop with Claude Code.
    pub fn run(&self, description: &str) -> BtcResult<PathBuf> {
        println!("Starting deep interview for: {}", description);

        let specs_dir = self.project_dir.join(".btc").join("specs");
        std::fs::create_dir_all(&specs_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let filename = format!("interview-{}.md", timestamp);
        let spec_path = specs_dir.join(&filename);

        let content = format!(
            "# Interview Spec\n\n\
             **Created:** {}\n\
             **Description:** {}\n\n\
             ## Requirements\n\n\
             _To be filled during deep interview._\n\n\
             ## Acceptance Criteria\n\n\
             _To be determined._\n",
            chrono::Utc::now().to_rfc3339(),
            description,
        );

        std::fs::write(&spec_path, content)?;
        Ok(spec_path)
    }
}
