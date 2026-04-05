use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dag::node::TaskStatus;
use crate::error::BtcResult;
use crate::types::TaskId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub node_id: TaskId,
    pub from_status: TaskStatus,
    pub to_status: TaskStatus,
    pub timestamp: DateTime<Utc>,
}

pub struct ExecutionJournal {
    path: PathBuf,
}

impl ExecutionJournal {
    pub fn new(path: PathBuf) -> BtcResult<Self> {
        if !path.exists() {
            File::create(&path)?;
        }
        Ok(Self { path })
    }

    pub fn record_transition(
        &self,
        node_id: &TaskId,
        from: TaskStatus,
        to: TaskStatus,
    ) -> BtcResult<()> {
        let entry = JournalEntry {
            node_id: node_id.clone(),
            from_status: from,
            to_status: to,
            timestamp: Utc::now(),
        };
        let line = serde_json::to_string(&entry)?;
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        writeln!(file, "{}", line)?;
        file.flush()?;
        file.sync_all()?;
        Ok(())
    }

    pub fn replay(&self) -> BtcResult<Vec<JournalEntry>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: JournalEntry = serde_json::from_str(&line)?;
            entries.push(entry);
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::node::TaskStatus;
    use tempfile::NamedTempFile;

    #[test]
    fn test_journal_record_and_replay() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let journal = ExecutionJournal::new(path).unwrap();
        let id = TaskId::from_str("test_node");

        journal
            .record_transition(&id, TaskStatus::Pending, TaskStatus::Ready)
            .unwrap();
        journal
            .record_transition(&id, TaskStatus::Ready, TaskStatus::Running)
            .unwrap();
        journal
            .record_transition(&id, TaskStatus::Running, TaskStatus::Complete)
            .unwrap();

        let entries = journal.replay().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].from_status, TaskStatus::Pending);
        assert_eq!(entries[0].to_status, TaskStatus::Ready);
        assert_eq!(entries[2].to_status, TaskStatus::Complete);
        assert_eq!(entries[0].node_id.0, "test_node");
    }
}
