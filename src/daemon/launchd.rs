use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{BtcError, BtcResult};

pub struct LaunchdManager;

impl LaunchdManager {
    /// Path to the launchd plist file.
    pub fn plist_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join("com.btc.daemon.plist")
    }

    /// Install the daemon as a launchd service.
    pub fn install(btc_binary: &Path) -> BtcResult<()> {
        let plist_path = Self::plist_path();
        let binary_str = btc_binary.to_string_lossy();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.btc.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary_str}</string>
        <string>daemon</string>
        <string>start</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{home}/.btc/logs/daemon.out.log</string>
    <key>StandardErrorPath</key>
    <string>{home}/.btc/logs/daemon.err.log</string>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#
        );

        // Ensure parent directory exists
        if let Some(parent) = plist_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Ensure log directory exists
        let log_dir = PathBuf::from(&home).join(".btc").join("logs");
        std::fs::create_dir_all(&log_dir)?;

        std::fs::write(&plist_path, plist_content)?;

        let status = Command::new("launchctl")
            .args(["load", &plist_path.to_string_lossy()])
            .status()?;

        if !status.success() {
            return Err(BtcError::Daemon("launchctl load failed".to_string()));
        }

        tracing::info!("Installed launchd service at {:?}", plist_path);
        Ok(())
    }

    /// Uninstall the daemon launchd service.
    pub fn uninstall() -> BtcResult<()> {
        let plist_path = Self::plist_path();

        if plist_path.exists() {
            let status = Command::new("launchctl")
                .args(["unload", &plist_path.to_string_lossy()])
                .status()?;

            if !status.success() {
                return Err(BtcError::Daemon("launchctl unload failed".to_string()));
            }

            std::fs::remove_file(&plist_path)?;
            tracing::info!("Uninstalled launchd service");
        }

        Ok(())
    }

    /// Check if the launchd service is installed.
    pub fn is_installed() -> bool {
        Self::plist_path().exists()
    }
}
