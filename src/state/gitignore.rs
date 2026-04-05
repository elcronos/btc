use std::fs;
use std::path::Path;

use crate::error::BtcResult;

pub struct GitignoreEnforcer;

impl GitignoreEnforcer {
    pub fn ensure_btc_ignored(project_dir: &Path) -> BtcResult<()> {
        let gitignore_path = project_dir.join(".gitignore");

        if gitignore_path.exists() {
            let contents = fs::read_to_string(&gitignore_path)?;
            if Self::has_btc_entry(&contents) {
                return Ok(());
            }
            // Append .btc/ entry
            let mut new_contents = contents;
            if !new_contents.ends_with('\n') {
                new_contents.push('\n');
            }
            new_contents.push_str("\n# BTC orchestrator state (contains tokens)\n.btc/\n");
            fs::write(&gitignore_path, new_contents)?;
        } else {
            let contents = "# BTC orchestrator state (contains tokens)\n.btc/\n";
            fs::write(&gitignore_path, contents)?;
        }

        Ok(())
    }

    pub fn verify_btc_ignored(project_dir: &Path) -> BtcResult<bool> {
        let gitignore_path = project_dir.join(".gitignore");
        if !gitignore_path.exists() {
            return Ok(false);
        }
        let contents = fs::read_to_string(&gitignore_path)?;
        Ok(Self::has_btc_entry(&contents))
    }

    fn has_btc_entry(contents: &str) -> bool {
        contents.lines().any(|line| {
            let trimmed = line.trim();
            matches!(trimmed, ".btc/" | ".btc" | "/.btc/")
        })
    }
}
