use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use super::node::{FileNode, NodeId, NodeType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    SizeDesc,
    SizeAsc,
    NameAsc,
    NameDesc,
    CountDesc,
    ModifiedDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryTree {
    pub root_id: NodeId,
    pub nodes: Vec<FileNode>,
    #[serde(skip)]
    pub path_to_id: HashMap<PathBuf, NodeId>,
}

impl DirectoryTree {
    pub fn new(root_path: PathBuf) -> Self {
        let root_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();

        let root_node = FileNode::new(0, root_name, root_path.clone(), NodeType::Directory, None);
        let mut path_to_id = HashMap::new();
        path_to_id.insert(root_path, 0);

        Self {
            root_id: 0,
            nodes: vec![root_node],
            path_to_id,
        }
    }

    pub fn rebuild_index(&mut self) {
        self.path_to_id.clear();
        for (idx, node) in self.nodes.iter().enumerate() {
            self.path_to_id.insert(node.path.clone(), idx);
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<&FileNode> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut FileNode> {
        self.nodes.get_mut(id)
    }

    pub fn get_id_by_path(&self, path: &Path) -> Option<NodeId> {
        self.path_to_id.get(path).copied()
    }

    /// Insert or update an entry from the scanner
    pub fn insert_entry(
        &mut self,
        path: PathBuf,
        node_type: NodeType,
        size: u64,
        modified: u64,
    ) -> NodeId {
        if let Some(&existing_id) = self.path_to_id.get(&path) {
            let node = &mut self.nodes[existing_id];
            node.apparent_size = size;
            node.disk_size = size;
            node.modified = modified;
            return existing_id;
        }

        let parent_id = path.parent().and_then(|p| self.path_to_id.get(p).copied());
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let new_id = self.nodes.len();
        let mut new_node = FileNode::new(new_id, name, path.clone(), node_type, parent_id);
        new_node.apparent_size = size;
        new_node.disk_size = size;
        new_node.modified = modified;

        self.nodes.push(new_node);
        self.path_to_id.insert(path, new_id);

        if let Some(pid) = parent_id {
            if let Some(parent_node) = self.nodes.get_mut(pid) {
                parent_node.children.push(new_id);
            }
        }

        new_id
    }

    /// Propagates directory sizes from leaves up to the root
    pub fn aggregate_sizes(&mut self) {
        // Collect node IDs in topological order (deepest depth first)
        let mut depths: Vec<(usize, NodeId)> = self
            .nodes
            .iter()
            .map(|n| (n.path.components().count(), n.id))
            .collect();

        // Sort descending by depth so children are processed before parents
        depths.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        // Reset directory aggregates
        for node in &mut self.nodes {
            if node.is_dir() {
                node.disk_size = 0;
                node.item_count = 0;
            }
        }

        for (_, id) in depths {
            let (is_dir, disk_size, parent_id) = {
                let n = &self.nodes[id];
                (n.is_dir(), n.disk_size, n.parent)
            };

            if let Some(pid) = parent_id {
                let parent = &mut self.nodes[pid];
                parent.disk_size += disk_size;
                parent.item_count += if is_dir { 1 } else { 1 };
            }
        }
    }

    /// Returns sorted child NodeIds according to chosen SortOrder
    pub fn sorted_children(&self, parent_id: NodeId, order: SortOrder) -> Vec<NodeId> {
        let parent = match self.get_node(parent_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut children = parent.children.clone();

        children.sort_by(|&a_id, &b_id| {
            let a = &self.nodes[a_id];
            let b = &self.nodes[b_id];

            // Keep directories on top unless sorting specifically by name/time
            if a.is_dir() != b.is_dir() {
                return b.is_dir().cmp(&a.is_dir());
            }

            match order {
                SortOrder::SizeDesc => b.disk_size.cmp(&a.disk_size),
                SortOrder::SizeAsc => a.disk_size.cmp(&b.disk_size),
                SortOrder::NameAsc => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortOrder::NameDesc => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
                SortOrder::CountDesc => b.item_count.cmp(&a.item_count),
                SortOrder::ModifiedDesc => b.modified.cmp(&a.modified),
            }
        });

        children
    }
}
