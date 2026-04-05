use std::path::PathBuf;
use std::process::Command;

use crate::error::{BtcError, BtcResult};
use crate::state::GitignoreEnforcer;

use super::HookInstaller;

pub struct SetupWizard {
    project_dir: PathBuf,
}

impl SetupWizard {
    pub fn new(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }

    pub fn run(&self) -> BtcResult<()> {
        println!("=== BTC Setup Wizard ===\n");

        // Step 1: Check for Claude Code
        if Self::check_claude_code() {
            println!("[OK] Claude Code detected.");
        } else {
            println!("[!!] Claude Code not found.");
            println!("     Install from: https://docs.anthropic.com/en/docs/claude-code");
            println!("     Run: npm install -g @anthropic-ai/claude-code");
            return Err(BtcError::ClaudeCodeNotFound);
        }

        // Step 2: Create .btc/ directory structure
        self.create_project_structure()?;
        println!("[OK] Project structure created.");

        // Step 3: Ensure .btc/ is in .gitignore
        GitignoreEnforcer::ensure_btc_ignored(&self.project_dir)?;
        println!("[OK] .gitignore updated.");

        // Step 4: Write default config
        self.write_default_config()?;
        println!("[OK] Default config.toml written.");

        // Step 5: Install hooks placeholder
        HookInstaller::install(&self.project_dir)?;
        println!("[OK] Hook directory initialized.");

        // Step 6: Write default skills
        self.write_default_skills()?;
        println!("[OK] Default skills written.");

        println!("\n=== Setup complete! ===");
        println!("Next steps:");
        println!("  btc new \"description of what to build\"");
        println!("  btc plan");
        println!("  btc run");

        Ok(())
    }

    pub fn check_claude_code() -> bool {
        Command::new("which")
            .arg("claude")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn create_project_structure(&self) -> BtcResult<()> {
        let btc_dir = self.project_dir.join(".btc");
        let subdirs = ["state", "specs", "plans", "skills", "hooks", "staging"];

        for sub in &subdirs {
            std::fs::create_dir_all(btc_dir.join(sub))?;
        }

        Ok(())
    }

    fn write_default_skills(&self) -> BtcResult<()> {
        let skills_dir = self.project_dir.join(".btc").join("skills");

        let skills = [
            (
                "refactor",
                r#"---
name: refactor
description: Refactor code for clarity and maintainability
triggers:
  - refactor
  - cleanup
  - clean up
---
Analyze the codebase and identify areas that need refactoring. Focus on reducing complexity, improving naming, extracting reusable functions, and eliminating duplication. Make changes incrementally and verify each step compiles.
"#,
            ),
            (
                "test",
                r#"---
name: test
description: Generate comprehensive test coverage
triggers:
  - test
  - coverage
  - testing
---
Analyze the codebase and write comprehensive tests. Cover happy paths, edge cases, and error conditions. Use the project's existing test framework and conventions. Run tests after writing to verify they pass.
"#,
            ),
            (
                "review",
                r#"---
name: review
description: Code review with security and quality focus
triggers:
  - review
  - audit
---
Review the codebase for code quality, security vulnerabilities, performance issues, and best practices. Provide actionable feedback with specific file and line references. Check for OWASP top 10, proper error handling, and input validation.
"#,
            ),
        ];

        for (name, content) in &skills {
            let path = skills_dir.join(format!("{}.md", name));
            if !path.exists() {
                std::fs::write(&path, content)?;
            }
        }

        Ok(())
    }

    fn write_default_config(&self) -> BtcResult<()> {
        let config_path = self.project_dir.join(".btc").join("config.toml");
        if config_path.exists() {
            println!("     config.toml already exists, skipping.");
            return Ok(());
        }

        let default_config = r#"# BTC Configuration
# See documentation for all options.

# [visual_qa]
# page_score_threshold = 90.0
# cross_page_threshold = 85.0
# max_retries = 3
# viewport = [1280, 720]

# [sandbox]
# enabled = true
# network_policy = "permissive"

# [limits]
# max_concurrent_agents = 4
# max_budget_usd = 10.0
# max_iterations = 20

# [remote.telegram]
# bot_token = ""
# allowed_chat_ids = []

# [remote.slack]
# app_token = ""
# bot_token = ""
# allowed_workspace_ids = []
# allowed_user_ids = []
"#;

        std::fs::write(&config_path, default_config)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_project_structure() {
        let tmp = TempDir::new().unwrap();
        let wizard = SetupWizard::new(tmp.path().to_path_buf());

        wizard.create_project_structure().unwrap();

        let btc_dir = tmp.path().join(".btc");
        assert!(btc_dir.join("state").is_dir());
        assert!(btc_dir.join("specs").is_dir());
        assert!(btc_dir.join("plans").is_dir());
        assert!(btc_dir.join("skills").is_dir());
        assert!(btc_dir.join("hooks").is_dir());
        assert!(btc_dir.join("staging").is_dir());
    }

    #[test]
    fn test_gitignore_called_during_setup() {
        let tmp = TempDir::new().unwrap();
        let wizard = SetupWizard::new(tmp.path().to_path_buf());

        // Create the structure first so the rest of setup can proceed
        wizard.create_project_structure().unwrap();

        // Call gitignore enforcer directly (as setup does)
        GitignoreEnforcer::ensure_btc_ignored(tmp.path()).unwrap();

        let gitignore = tmp.path().join(".gitignore");
        assert!(gitignore.exists());
        let contents = std::fs::read_to_string(&gitignore).unwrap();
        assert!(contents.contains(".btc/"));
    }
}
