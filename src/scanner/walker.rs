use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crossbeam_channel::Sender;
use walkdir::WalkDir;

use crate::model::NodeType;

#[derive(Debug, Clone)]
pub struct ScannedEntry {
    pub path: PathBuf,
    pub node_type: NodeType,
    pub size: u64,
    pub modified: u64,
}

#[derive(Debug, Clone)]
pub enum ScannerMessage {
    Batch(Vec<ScannedEntry>),
    Progress { files: usize, bytes: u64, current_path: PathBuf },
    Complete { total_files: usize, total_bytes: u64 },
    Error(String),
}

pub struct Scanner;

impl Scanner {
    /// Launches a non-blocking background filesystem scan
    pub fn start_scan(
        root_path: PathBuf,
        cancel_token: Arc<AtomicBool>,
        tx: Sender<ScannerMessage>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut visited_inodes: HashSet<(u64, u64)> = HashSet::new();
            let mut batch: Vec<ScannedEntry> = Vec::with_capacity(512);
            let mut total_files: usize = 0;
            let mut total_bytes: u64 = 0;
            let mut last_progress = SystemTime::now();

            let walker = WalkDir::new(&root_path)
                .follow_links(false)
                .same_file_system(false)
                .into_iter();

            for entry in walker {
                if cancel_token.load(Ordering::Relaxed) {
                    break;
                }

                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        // Skip permission denied or dangling links gracefully
                        let _ = tx.send(ScannerMessage::Error(err.to_string()));
                        continue;
                    }
                };

                let path = entry.path().to_path_buf();
                let file_type = entry.file_type();

                let metadata = match fs::symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Cycle detection for inodes
                let dev = metadata.dev();
                let ino = metadata.ino();
                if !visited_inodes.insert((dev, ino)) {
                    continue; // Skip duplicate / looped inodes
                }

                // In Unix, metadata.blocks() * 512 is true physical allocated disk size
                let physical_size = metadata.blocks() * 512;
                let size = if physical_size > 0 { physical_size } else { metadata.len() };

                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let node_type = if file_type.is_dir() {
                    NodeType::Directory
                } else if file_type.is_symlink() {
                    NodeType::Symlink
                } else {
                    total_bytes += size;
                    NodeType::File
                };

                total_files += 1;

                batch.push(ScannedEntry {
                    path: path.clone(),
                    node_type,
                    size,
                    modified,
                });

                if batch.len() >= 512 {
                    let _ = tx.send(ScannerMessage::Batch(std::mem::take(&mut batch)));
                }

                // Send throttled progress heartbeat every 50ms
                if let Ok(elapsed) = last_progress.elapsed() {
                    if elapsed >= Duration::from_millis(50) {
                        let _ = tx.send(ScannerMessage::Progress {
                            files: total_files,
                            bytes: total_bytes,
                            current_path: path,
                        });
                        last_progress = SystemTime::now();
                    }
                }
            }

            // Flush remaining batch
            if !batch.is_empty() {
                let _ = tx.send(ScannerMessage::Batch(batch));
            }

            let _ = tx.send(ScannerMessage::Complete {
                total_files,
                total_bytes,
            });
        })
    }
}
