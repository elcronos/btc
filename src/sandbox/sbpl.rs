use std::path::{Path, PathBuf};

use crate::config::NetworkPolicy;
use crate::error::BtcResult;

pub struct SbplGenerator;

impl SbplGenerator {
    /// Generate a macOS SBPL (Sandbox Profile Language) string.
    pub fn generate(project_dir: &Path, network_policy: NetworkPolicy) -> String {
        let project_path = project_dir.display();

        let network_rules = match network_policy {
            NetworkPolicy::Permissive => "(allow network-outbound)".to_string(),
            NetworkPolicy::Strict => [
                "(allow network-outbound",
                "    (remote tcp \"api.anthropic.com:443\"))",
            ]
            .join("\n"),
        };

        format!(
            r#"(version 1)
(deny default)

;; Allow read/write within project directory
(allow file-read* file-write*
    (subpath "{project_path}"))

;; Allow read from system paths
(allow file-read*
    (subpath "/usr")
    (subpath "/bin")
    (subpath "/System")
    (subpath "/Library")
    (subpath "/private")
    (subpath "/dev"))

;; Allow process execution and forking
(allow process-exec)
(allow process-fork)
(allow sysctl-read)
(allow mach-lookup)

;; Network policy
{network_rules}
"#
        )
    }

    /// Generate the SBPL profile and write it to `.btc/sandbox.sb`.
    pub fn write_profile(
        project_dir: &Path,
        network_policy: NetworkPolicy,
    ) -> BtcResult<PathBuf> {
        let profile = Self::generate(project_dir, network_policy);
        let btc_dir = project_dir.join(".btc");
        std::fs::create_dir_all(&btc_dir)?;
        let profile_path = btc_dir.join("sandbox.sb");
        std::fs::write(&profile_path, profile)?;
        tracing::debug!(path = %profile_path.display(), "Wrote SBPL profile");
        Ok(profile_path)
    }
}
