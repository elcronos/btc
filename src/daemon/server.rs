use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::error::BtcResult;

use super::types::{DaemonCommand, DaemonResponse, DaemonStatus};

pub struct DaemonServer {
    socket_path: PathBuf,
}

impl DaemonServer {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Start listening for incoming daemon commands on the Unix socket.
    ///
    // TODO: verify LOCAL_PEERCRED on connections
    pub async fn listen(&self) -> BtcResult<()> {
        // Remove stale socket file if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("Daemon listening on {:?}", self.socket_path);

        loop {
            let (stream, _addr) = listener.accept().await?;
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut buf_reader = BufReader::new(reader);
                let mut line = String::new();

                if buf_reader.read_line(&mut line).await.is_ok() {
                    let response = match serde_json::from_str::<DaemonCommand>(&line) {
                        Ok(cmd) => Self::handle_command(cmd),
                        Err(e) => DaemonResponse::Error(format!("Invalid command: {e}")),
                    };

                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = writer.write_all(json.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                    }
                }
            });
        }
    }

    fn handle_command(cmd: DaemonCommand) -> DaemonResponse {
        match cmd {
            DaemonCommand::Status => DaemonResponse::Status(DaemonStatus {
                running: true,
                active_agents: 0,
                total_cost: 0.0,
                uptime_secs: 0.0,
                cron_jobs: 0,
            }),
            _ => DaemonResponse::Error("not implemented".to_string()),
        }
    }
}
