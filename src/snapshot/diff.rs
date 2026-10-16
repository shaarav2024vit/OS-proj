use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::model::DirectoryTree;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotItem {
    pub relative_path: PathBuf,
    pub is_dir: bool,
    pub disk_size: u64,
    pub modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: u64,
    pub root_path: PathBuf,
    pub total_size: u64,
    pub total_items: usize,
    pub items: Vec<SnapshotItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffKind {
    Added,
    Removed,
    Grown,
    Shrunk,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub relative_path: PathBuf,
    pub is_dir: bool,
    pub kind: DiffKind,
    pub old_size: u64,
    pub new_size: u64,
    pub size_delta: i64,
}

pub struct SnapshotManager;

impl SnapshotManager {
    pub fn snapshot_dir() -> PathBuf {
        directories::ProjectDirs::from("org", "vit", "strata")
            .map(|proj| proj.data_local_dir().join("snapshots"))
            .unwrap_or_else(|| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".local/share/strata/snapshots"))
                    .unwrap_or_else(|_| PathBuf::from("/tmp/strata_snapshots"))
            })
    }

    /// Capture a snapshot from an in-memory DirectoryTree
    pub fn create_snapshot(tree: &DirectoryTree, root_path: &Path) -> Result<Snapshot, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = format!("snap_{}", now);
        let mut items = Vec::new();

        for node in &tree.nodes {
            if let Ok(rel) = node.path.strip_prefix(root_path) {
                items.push(SnapshotItem {
                    relative_path: rel.to_path_buf(),
                    is_dir: node.is_dir(),
                    disk_size: node.disk_size,
                    modified: node.modified,
                });
            }
        }

        let root_node = tree.get_node(tree.root_id);
        let total_size = root_node.map(|n| n.disk_size).unwrap_or(0);
        let total_items = tree.nodes.len();

        let snapshot = Snapshot {
            id: id.clone(),
            created_at: now,
            root_path: root_path.to_path_buf(),
            total_size,
            total_items,
            items,
        };

        let dir = Self::snapshot_dir();
        fs::create_dir_all(&dir)?;
        let snap_file = dir.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&snapshot)?;
        fs::write(snap_file, content)?;

        Ok(snapshot)
    }

    /// List all saved snapshots
    pub fn list_snapshots() -> Vec<Snapshot> {
        let dir = Self::snapshot_dir();
        let mut list = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(snap) = serde_json::from_str::<Snapshot>(&content) {
                            list.push(snap);
                        }
                    }
                }
            }
        }

        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    /// Perform a high-speed $O(N + M)$ diff between two snapshots
    pub fn diff(old: &Snapshot, new: &Snapshot) -> Vec<DiffEntry> {
        let mut old_map: HashMap<&PathBuf, &SnapshotItem> = HashMap::new();
        for item in &old.items {
            old_map.insert(&item.relative_path, item);
        }

        let mut new_map: HashMap<&PathBuf, &SnapshotItem> = HashMap::new();
        for item in &new.items {
            new_map.insert(&item.relative_path, item);
        }

        let mut diffs: Vec<DiffEntry> = Vec::new();

        // Check new items for Added, Grown, Shrunk, Unchanged
        for (path, &new_item) in &new_map {
            if let Some(&old_item) = old_map.get(path) {
                let size_delta = (new_item.disk_size as i64) - (old_item.disk_size as i64);
                let kind = if size_delta > 0 {
                    DiffKind::Grown
                } else if size_delta < 0 {
                    DiffKind::Shrunk
                } else {
                    DiffKind::Unchanged
                };

                diffs.push(DiffEntry {
                    relative_path: (*path).clone(),
                    is_dir: new_item.is_dir,
                    kind,
                    old_size: old_item.disk_size,
                    new_size: new_item.disk_size,
                    size_delta,
                });
            } else {
                diffs.push(DiffEntry {
                    relative_path: (*path).clone(),
                    is_dir: new_item.is_dir,
                    kind: DiffKind::Added,
                    old_size: 0,
                    new_size: new_item.disk_size,
                    size_delta: new_item.disk_size as i64,
                });
            }
        }

        // Check old items for Removed
        for (path, &old_item) in &old_map {
            if !new_map.contains_key(path) {
                diffs.push(DiffEntry {
                    relative_path: (*path).clone(),
                    is_dir: old_item.is_dir,
                    kind: DiffKind::Removed,
                    old_size: old_item.disk_size,
                    new_size: 0,
                    size_delta: -(old_item.disk_size as i64),
                });
            }
        }

        // Sort diffs: directories first, then by largest absolute size change
        diffs.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                b.size_delta.abs().cmp(&a.size_delta.abs())
            }
        });

        diffs
    }

    /// Export differential snapshot analysis to a clean Markdown report
    pub fn export_markdown_report(
        old_snap: &Snapshot,
        new_snap: &Snapshot,
        diffs: &[DiffEntry],
        target_file: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        use std::io::Write;
        let mut f = fs::File::create(target_file)?;

        writeln!(f, "# Strata: File System Differential Analysis Report")?;
        writeln!(f, "* **Target Directory**: `{}`", new_snap.root_path.display())?;
        writeln!(f, "* **Baseline Snapshot**: `{}`", old_snap.id)?;
        writeln!(f, "* **Compared Snapshot**: `{}`", new_snap.id)?;
        writeln!(f, "* **Total Files Analyzed**: {}", new_snap.total_items)?;
        writeln!(f, "\n---\n")?;
        writeln!(f, "## Detected Storage Changes\n")?;
        writeln!(f, "| Status | Type | Path | Delta | Previous Size | New Size |")?;
        writeln!(f, "| :---: | :---: | :--- | :---: | :---: | :---: |")?;

        for d in diffs {
            let status = match d.kind {
                DiffKind::Added => "✅ ADDED",
                DiffKind::Removed => "❌ REMOVED",
                DiffKind::Grown => "🔺 GROWN",
                DiffKind::Shrunk => "🔻 SHRUNK",
                DiffKind::Unchanged => "⏸️ UNCHANGED",
            };
            let kind = if d.is_dir { "📁 Dir" } else { "📄 File" };
            let delta = if d.size_delta > 0 {
                format!("+{}", format_bytes_report(d.size_delta as u64))
            } else if d.size_delta < 0 {
                format!("-{}", format_bytes_report(d.size_delta.unsigned_abs()))
            } else {
                "0 B".into()
            };

            writeln!(
                f,
                "| {} | {} | `{}` | **{}** | {} | {} |",
                status,
                kind,
                d.relative_path.display(),
                delta,
                format_bytes_report(d.old_size),
                format_bytes_report(d.new_size)
            )?;
        }

        writeln!(f, "\n*Report generated by Strata v0.1.0 (BCSE303P OS Lab)*")?;
        Ok(target_file.to_path_buf())
    }
}

fn format_bytes_report(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
