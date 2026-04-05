use std::path::Path;

use crate::error::BtcResult;

pub struct HookInstaller;

impl HookInstaller {
    /// Install hook configuration placeholder.
    pub fn install(project_dir: &Path) -> BtcResult<()> {
        let hooks_dir = project_dir.join(".btc").join("hooks");
        std::fs::create_dir_all(&hooks_dir)?;

        let readme_path = hooks_dir.join("README.md");
        if !readme_path.exists() {
            let contents = r#"# BTC Hooks

Place hook scripts in this directory. Hooks run at specific lifecycle points:

- `pre-plan.sh`  — runs before plan generation
- `post-plan.sh` — runs after plan generation
- `pre-run.sh`   — runs before orchestration execution
- `post-run.sh`  — runs after orchestration completes
- `on-error.sh`  — runs when an agent encounters an error

Each hook receives environment variables:
- `BTC_PROJECT_DIR` — project root
- `BTC_PHASE`       — current lifecycle phase
- `BTC_AGENT_ID`    — agent ID (if applicable)

Hooks must be executable (`chmod +x`).
"#;
            std::fs::write(&readme_path, contents)?;
        }

        Ok(())
    }

    /// Check if hooks directory exists.
    pub fn is_installed(project_dir: &Path) -> bool {
        project_dir.join(".btc").join("hooks").exists()
    }
}
