use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The global sync manifest stored at the root of the S3 prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManifest {
    pub version: u32,
    pub device_id: String,
    #[serde(default)]
    pub databases: BTreeMap<String, DbEntry>,
}

/// Metadata for a single synced database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbEntry {
    pub last_modified: DateTime<Utc>,
    pub checksum: String,
    pub size_bytes: u64,
}

impl SyncManifest {
    pub fn new(device_id: String) -> Self {
        Self {
            version: 1,
            device_id,
            databases: BTreeMap::new(),
        }
    }

    /// Update (or insert) an entry for a given database.
    pub fn upsert(&mut self, name: &str, checksum: String, size_bytes: u64) {
        self.databases.insert(
            name.to_string(),
            DbEntry {
                last_modified: Utc::now(),
                checksum,
                size_bytes,
            },
        );
    }

    /// Remove a database from the manifest.
    #[allow(dead_code)]
    pub fn remove(&mut self, name: &str) {
        self.databases.remove(name);
    }
}

/// The result of comparing a local database against the remote manifest.
#[derive(Debug, PartialEq, Eq)]
pub enum SyncState {
    /// Only exists locally, not yet pushed.
    LocalOnly,
    /// Only exists remotely, not yet pulled.
    RemoteOnly,
    /// Both exist with the same checksum — in sync.
    InSync,
    /// Local has changed (checksum differs, local is newer).
    LocalNewer,
    /// Remote has changed (checksum differs, remote is newer).
    RemoteNewer,
    /// Both have changed (diverged — conflict).
    #[allow(dead_code)]
    Diverged,
}

/// Compare local state against the remote manifest entry.
///
/// Returns a `SyncState` indicating what action should be taken.
#[allow(dead_code)]
pub fn compare(
    local_checksum: &str,
    remote_entry: Option<&DbEntry>,
    local_modified: Option<DateTime<Utc>>,
) -> SyncState {
    match remote_entry {
        None => SyncState::LocalOnly,
        Some(remote) => {
            // Same checksum = in sync (regardless of timestamps).
            if local_checksum == remote.checksum {
                return SyncState::InSync;
            }

            // Checksums differ — compare timestamps.
            match local_modified {
                Some(local_ts) if local_ts > remote.last_modified => {
                    // Remote might also have changed? Check if remote is new too.
                    // If remote.last_modified > last local manifest record, it diverged.
                    // But since we only have one timestamp each, just compare.
                    SyncState::LocalNewer
                }
                Some(_) => SyncState::RemoteNewer,
                None => {
                    // Can't determine local time; treat remote as newer.
                    SyncState::RemoteNewer
                }
            }
        }
    }
}
