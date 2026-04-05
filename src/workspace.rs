use std::path::Path;
use std::process::Command;
use crate::error::{BtcError, BtcResult};

pub fn launch_workspace(project_dir: &Path) -> BtcResult<()> {
    // Check tmux is available
    let tmux_check = Command::new("tmux").arg("-V").output();
    if tmux_check.is_err() || !tmux_check.unwrap().status.success() {
        return Err(BtcError::Other("tmux is required for workspace mode. Install with: brew install tmux".into()));
    }

    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("btc");
    let session_name = format!("btc-{}", project_name);
    let dir = project_dir.display().to_string();

    // Check if session already exists
    let existing = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output();

    if let Ok(output) = existing {
        if output.status.success() {
            // Attach to existing session
            println!("Attaching to existing workspace: {}", session_name);
            let status = Command::new("tmux")
                .args(["attach-session", "-t", &session_name])
                .status()
                .map_err(|e| BtcError::Other(format!("Failed to attach tmux: {}", e)))?;
            if !status.success() {
                return Err(BtcError::Other("Failed to attach to tmux session".into()));
            }
            return Ok(());
        }
    }

    // Create new tmux session with the BTC menu in the first pane
    // The session starts detached so we can set up all panes first
    Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s", &session_name,
            "-c", &dir,
            "-x", "200",
            "-y", "50",
        ])
        .status()
        .map_err(|e| BtcError::Other(format!("Failed to create tmux session: {}", e)))?;

    // Split right pane (60% width)
    Command::new("tmux")
        .args([
            "split-window",
            "-t", &session_name,
            "-h",
            "-p", "60",
            "-c", &dir,
        ])
        .status()
        .map_err(|e| BtcError::Other(format!("Failed to split pane: {}", e)))?;

    // Split right pane vertically (bottom half for dashboard)
    Command::new("tmux")
        .args([
            "split-window",
            "-t", &format!("{}:0.1", session_name),
            "-v",
            "-p", "40",
            "-c", &dir,
        ])
        .status()
        .map_err(|e| BtcError::Other(format!("Failed to split pane: {}", e)))?;

    // Pane 0 (left): BTC interactive menu
    Command::new("tmux")
        .args([
            "send-keys",
            "-t", &format!("{}:0.0", session_name),
            "btc", "Enter",
        ])
        .status()
        .ok();

    // Pane 1 (right-top): Claude Code with bypass permissions
    Command::new("tmux")
        .args([
            "send-keys",
            "-t", &format!("{}:0.1", session_name),
            "claude --permission-mode bypassPermissions",
            "Enter",
        ])
        .status()
        .ok();

    // Pane 2 (right-bottom): BTC dashboard
    Command::new("tmux")
        .args([
            "send-keys",
            "-t", &format!("{}:0.2", session_name),
            "btc dash", "Enter",
        ])
        .status()
        .ok();

    // Set pane titles
    Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.0", session_name), "-T", "BTC Menu"])
        .status().ok();
    Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.1", session_name), "-T", "Claude Code"])
        .status().ok();
    Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.2", session_name), "-T", "Dashboard"])
        .status().ok();

    // Enable pane border titles
    Command::new("tmux")
        .args(["set-option", "-t", &session_name, "pane-border-status", "top"])
        .status().ok();
    Command::new("tmux")
        .args(["set-option", "-t", &session_name, "pane-border-format", " #{pane_title} "])
        .status().ok();

    // Style the panes
    Command::new("tmux")
        .args(["set-option", "-t", &session_name, "pane-active-border-style", "fg=cyan"])
        .status().ok();
    Command::new("tmux")
        .args(["set-option", "-t", &session_name, "pane-border-style", "fg=colour240"])
        .status().ok();

    // Set status bar
    Command::new("tmux")
        .args(["set-option", "-t", &session_name, "status-style", "bg=colour235,fg=cyan"])
        .status().ok();
    Command::new("tmux")
        .args(["set-option", "-t", &session_name, "status-left", &format!(" BTC | {} ", project_name)])
        .status().ok();
    Command::new("tmux")
        .args(["set-option", "-t", &session_name, "status-right", " %H:%M "])
        .status().ok();

    // Focus on the BTC menu pane
    Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.0", session_name)])
        .status().ok();

    // Attach to the session
    println!("Launching BTC workspace: {}", session_name);
    let status = Command::new("tmux")
        .args(["attach-session", "-t", &session_name])
        .status()
        .map_err(|e| BtcError::Other(format!("Failed to attach tmux: {}", e)))?;

    if !status.success() {
        return Err(BtcError::Other("Failed to attach to tmux session".into()));
    }

    Ok(())
}
