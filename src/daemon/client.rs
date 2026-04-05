use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::error::BtcResult;

use super::types::{DaemonCommand, DaemonResponse};

pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Send a command to the daemon and return the response.
    pub async fn send(&self, cmd: DaemonCommand) -> BtcResult<DaemonResponse> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let (reader, mut writer) = stream.into_split();

        let json = serde_json::to_string(&cmd)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;

        let response: DaemonResponse = serde_json::from_str(&line)?;
        Ok(response)
    }

    /// Check if the daemon is currently running by testing socket connectivity.
    pub fn is_daemon_running(&self) -> bool {
        self.socket_path.exists()
    }
}
