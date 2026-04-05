use std::path::Path;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::error::{BtcError, BtcResult};
use crate::sandbox::Sandbox;

use super::stream::{parse_line, StreamEvent};

/// Optional parameters for spawning a Claude Code subprocess.
#[derive(Debug, Default)]
pub struct SpawnOptions<'a> {
    pub model: Option<&'a str>,
    pub system_prompt: Option<&'a str>,
    pub extra_flags: Vec<String>,
}

/// Wraps a `tokio::process::Child` running `claude` in streaming mode.
pub struct ClaudeProcess {
    child: Child,
    reader: BufReader<tokio::process::ChildStdout>,
}

impl ClaudeProcess {
    /// Spawn a Claude Code subprocess with the given prompt.
    ///
    /// The command is built as:
    /// ```text
    /// claude -p "<prompt>" --output-format stream-json --bare [--model <m>] [--system-prompt <s>] [extra_flags...]
    /// ```
    ///
    /// The sandbox's `wrap_command` is called on the `Command` before spawning.
    pub async fn spawn(
        prompt: &str,
        sandbox: &dyn Sandbox,
        project_dir: &Path,
        opts: SpawnOptions<'_>,
    ) -> BtcResult<Self> {
        let mut cmd = Command::new("claude");

        cmd.arg("-p")
            .arg(prompt)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--bare");

        if let Some(model) = opts.model {
            cmd.arg("--model").arg(model);
        }

        if let Some(sys) = opts.system_prompt {
            cmd.arg("--system-prompt").arg(sys);
        }

        for flag in &opts.extra_flags {
            cmd.arg(flag);
        }

        cmd.current_dir(project_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());

        sandbox.wrap_command(&mut cmd, project_dir)?;

        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BtcError::ClaudeCodeNotFound
            } else {
                BtcError::SubprocessFailed(format!("failed to spawn claude: {e}"))
            }
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            BtcError::SubprocessFailed("failed to capture stdout".to_string())
        })?;

        let reader = BufReader::new(stdout);

        tracing::debug!(
            sandbox = sandbox.name(),
            project_dir = %project_dir.display(),
            "Claude process spawned"
        );

        Ok(Self { child, reader })
    }

    /// Read the next NDJSON event from stdout.
    ///
    /// Returns `Ok(None)` when the stream has ended (EOF).
    pub async fn read_event(&mut self) -> BtcResult<Option<StreamEvent>> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = self.reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                return Ok(None); // EOF
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match parse_line(trimmed) {
                Ok(event) => return Ok(Some(event)),
                Err(e) => {
                    tracing::warn!("Skipping malformed line: {e}");
                    continue;
                }
            }
        }
    }

    /// Send SIGTERM, then SIGKILL after the timeout elapses.
    pub async fn kill(&mut self, timeout: Duration) -> BtcResult<()> {
        #[cfg(unix)]
        {
            if let Some(pid) = self.child.id() {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let nix_pid = Pid::from_raw(pid as i32);
                let _ = kill(nix_pid, Signal::SIGTERM);

                tokio::select! {
                    _ = self.child.wait() => {
                        return Ok(());
                    }
                    _ = tokio::time::sleep(timeout) => {
                        let _ = kill(nix_pid, Signal::SIGKILL);
                        let _ = self.child.wait().await;
                    }
                }
            }
        }

        #[cfg(not(unix))]
        {
            let _ = timeout;
            self.child.kill().await?;
        }

        Ok(())
    }

    /// Check whether the subprocess is still running.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}
