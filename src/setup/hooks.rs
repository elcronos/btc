use std::path::Path;

use crate::error::BtcResult;

pub struct HookInstaller;

impl HookInstaller {
    /// Install hook configuration placeholder and event logging script.
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

        // Write the event logging hook script
        let hook_script = "#!/bin/bash\nEVENT_LOG=\"$(pwd)/.btc/agent-events.jsonl\"\ncat | jq -c '. + {\"timestamp\": (now | todate)}' >> \"$EVENT_LOG\" 2>/dev/null\n";
        let script_path = hooks_dir.join("on-event.sh");
        if !script_path.exists() {
            std::fs::write(&script_path, hook_script)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
            }
        }

        Ok(())
    }

    /// Check if hooks directory exists.
    pub fn is_installed(project_dir: &Path) -> bool {
        project_dir.join(".btc").join("hooks").exists()
    }
}
