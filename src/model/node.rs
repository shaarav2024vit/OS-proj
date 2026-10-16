use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub type NodeId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Directory,
    File,
    Symlink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub id: NodeId,
    pub name: String,
    pub path: PathBuf,
    pub node_type: NodeType,
    pub apparent_size: u64,
    pub disk_size: u64,         // Aggregated for directories
    pub item_count: usize,      // Number of files/dirs inside
    pub modified: u64,          // Unix timestamp
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub is_encrypted: bool,     // Ends with .strata
}

impl FileNode {
    pub fn new(id: NodeId, name: String, path: PathBuf, node_type: NodeType, parent: Option<NodeId>) -> Self {
        let is_encrypted = name.ends_with(".strata");
        Self {
            id,
            name,
            path,
            node_type,
            apparent_size: 0,
            disk_size: 0,
            item_count: 0,
            modified: 0,
            parent,
            children: Vec::new(),
            is_encrypted,
        }
    }

    pub fn is_dir(&self) -> bool {
        self.node_type == NodeType::Directory
    }
}
