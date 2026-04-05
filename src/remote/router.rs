use crate::daemon::types::DaemonCommand;
use crate::error::{BtcError, BtcResult};

use super::auth::AuthValidator;

/// Characters that are rejected as potential shell metacharacters.
const SHELL_METACHARACTERS: &[char] = &['$', '`', '|', ';', '&', '(', ')', '{', '}', '<', '>'];

pub struct CommandRouter {
    auth: AuthValidator,
}

impl CommandRouter {
    pub fn new(auth: AuthValidator) -> Self {
        Self { auth }
    }

    /// Parse a text command into a DaemonCommand.
    pub fn parse(text: &str) -> BtcResult<DaemonCommand> {
        let trimmed = text.trim();

        // Reject shell metacharacters
        if trimmed.chars().any(|c| SHELL_METACHARACTERS.contains(&c)) {
            return Err(BtcError::Remote(
                "Command contains forbidden shell metacharacters".to_string(),
            ));
        }

        let lower = trimmed.to_lowercase();
        match lower.as_str() {
            "status" => Ok(DaemonCommand::Status),
            "pause" => Ok(DaemonCommand::Pause),
            "resume" => Ok(DaemonCommand::Resume),
            "approve" => Ok(DaemonCommand::Approve),
            "cancel" => Ok(DaemonCommand::Cancel),
            _ if lower.starts_with("run ") => {
                let skill = trimmed[4..].trim().to_string();
                if skill.is_empty() {
                    return Err(BtcError::Remote("Missing skill name for run command".to_string()));
                }
                Ok(DaemonCommand::Run(skill))
            }
            _ => Err(BtcError::Remote(format!("Unknown command: {trimmed}"))),
        }
    }

    /// Parse a command with authentication checks.
    pub fn parse_with_auth(
        &self,
        text: &str,
        sender_id: &str,
        channel: &str,
    ) -> BtcResult<DaemonCommand> {
        let authorized = match channel {
            "telegram" => {
                let chat_id: i64 = sender_id
                    .parse()
                    .map_err(|_| BtcError::Remote("Invalid telegram chat ID".to_string()))?;
                self.auth.validate_telegram(chat_id)
            }
            "slack" => self.auth.validate_slack(sender_id),
            _ => false,
        };

        if !authorized {
            return Err(BtcError::Remote(format!(
                "Unauthorized sender: {sender_id} on {channel}"
            )));
        }

        Self::parse(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        let cmd = CommandRouter::parse("status").unwrap();
        assert!(matches!(cmd, DaemonCommand::Status));
    }

    #[test]
    fn test_parse_pause() {
        let cmd = CommandRouter::parse("pause").unwrap();
        assert!(matches!(cmd, DaemonCommand::Pause));
    }

    #[test]
    fn test_parse_resume() {
        let cmd = CommandRouter::parse("resume").unwrap();
        assert!(matches!(cmd, DaemonCommand::Resume));
    }

    #[test]
    fn test_parse_approve() {
        let cmd = CommandRouter::parse("approve").unwrap();
        assert!(matches!(cmd, DaemonCommand::Approve));
    }

    #[test]
    fn test_parse_cancel() {
        let cmd = CommandRouter::parse("cancel").unwrap();
        assert!(matches!(cmd, DaemonCommand::Cancel));
    }

    #[test]
    fn test_parse_run() {
        let cmd = CommandRouter::parse("run deploy").unwrap();
        assert!(matches!(cmd, DaemonCommand::Run(ref s) if s == "deploy"));
    }

    #[test]
    fn test_parse_run_case_insensitive() {
        let cmd = CommandRouter::parse("RUN MySkill").unwrap();
        assert!(matches!(cmd, DaemonCommand::Run(ref s) if s == "MySkill"));
    }

    #[test]
    fn test_rejects_shell_metacharacters() {
        assert!(CommandRouter::parse("run foo; rm -rf /").is_err());
        assert!(CommandRouter::parse("status | grep").is_err());
        assert!(CommandRouter::parse("run $(whoami)").is_err());
        assert!(CommandRouter::parse("run `id`").is_err());
        assert!(CommandRouter::parse("run foo & bar").is_err());
    }

    #[test]
    fn test_parse_unknown_command() {
        assert!(CommandRouter::parse("foobar").is_err());
    }
}
