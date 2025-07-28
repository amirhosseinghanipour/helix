use crate::core::commit::FileChange;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexNode {
    File(IndexEntry),
    Directory(HashMap<String, IndexNode>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub entries: HashMap<String, IndexNode>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: String,
    pub content_hash: String,
    pub size: u64,
    pub mode: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub stage: u32,
}

impl IndexNode {
    pub fn as_directory_mut(&mut self) -> Option<&mut HashMap<String, IndexNode>> {
        match self {
            IndexNode::Directory(ref mut map) => Some(map),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn as_directory(&self) -> Option<&HashMap<String, IndexNode>> {
        match self {
            IndexNode::Directory(ref map) => Some(map),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn as_file(&self) -> Option<&IndexEntry> {
        match self {
            IndexNode::File(ref entry) => Some(entry),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn as_file_mut(&mut self) -> Option<&mut IndexEntry> {
        match self {
            IndexNode::File(ref mut entry) => Some(entry),
            _ => None,
        }
    }
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: 2,
        }
    }

    pub fn add_file(&mut self, path: &str, entry: IndexEntry) {
        let parts: Vec<&str> = path.split('/').collect();
        let mut node = &mut self.entries;
        for part in &parts[..parts.len() - 1] {
            node = node
                .entry(part.to_string())
                .or_insert_with(|| IndexNode::Directory(HashMap::new()))
                .as_directory_mut()
                .unwrap();
        }
        node.insert(parts.last().unwrap().to_string(), IndexNode::File(entry));
    }

    #[allow(dead_code)]
    pub fn remove_file(&mut self, path: &str) {
        let parts: Vec<&str> = path.split('/').collect();
        let mut node = &mut self.entries;
        for part in &parts[..parts.len() - 1] {
            if let Some(IndexNode::Directory(ref mut map)) = node.get_mut(*part) {
                node = map;
            } else {
                return;
            }
        }
        node.remove(*parts.last().unwrap());
    }

    #[allow(dead_code)]
    pub fn get_file(&self, path: &str) -> Option<&IndexEntry> {
        let parts: Vec<&str> = path.split('/').collect();
        let mut node = &self.entries;
        for part in &parts[..parts.len() - 1] {
            if let Some(IndexNode::Directory(map)) = node.get(*part) {
                node = map;
            } else {
                return None;
            }
        }
        node.get(*parts.last().unwrap()).and_then(|n| n.as_file())
    }

    #[allow(dead_code)]
    pub fn has_file(&self, path: &str) -> bool {
        self.get_file(path).is_some()
    }

    pub fn get_all_files(&self) -> Vec<&IndexEntry> {
        fn collect_files<'a>(
            node: &'a HashMap<String, IndexNode>,
            files: &mut Vec<&'a IndexEntry>,
        ) {
            for n in node.values() {
                match n {
                    IndexNode::File(entry) => files.push(entry),
                    IndexNode::Directory(map) => collect_files(map, files),
                }
            }
        }
        let mut files = Vec::new();
        collect_files(&self.entries, &mut files);
        files
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.get_all_files().len()
    }

    #[allow(dead_code)]
    fn calculate_hash(content: &[u8]) -> String {
        crate::utils::hash_utils::calculate_hash(content)
    }

    pub fn to_file_changes(&self) -> HashMap<String, FileChange> {
        fn collect_changes(
            node: &HashMap<String, IndexNode>,
            changes: &mut HashMap<String, FileChange>,
        ) {
            for (_name, n) in node {
                match n {
                    IndexNode::File(entry) => {
                        changes.insert(
                            entry.path.clone(),
                            FileChange::new(
                                entry.path.clone(),
                                crate::core::commit::ChangeType::Added,
                                entry.content_hash.clone(),
                                entry.size,
                                entry.mode,
                            ),
                        );
                    }
                    IndexNode::Directory(map) => collect_changes(map, changes),
                }
            }
        }
        let mut changes = HashMap::new();
        collect_changes(&self.entries, &mut changes);
        changes
    }

    pub fn get_staged_files(&self) -> Vec<&IndexEntry> {
        fn collect_files<'a>(
            node: &'a HashMap<String, IndexNode>,
            files: &mut Vec<&'a IndexEntry>,
        ) {
            for n in node.values() {
                match n {
                    IndexNode::File(entry) => files.push(entry),
                    IndexNode::Directory(map) => collect_files(map, files),
                }
            }
        }
        let mut files = Vec::new();
        collect_files(&self.entries, &mut files);
        files
    }

    pub fn get_file_paths(&self) -> Vec<String> {
        fn collect_paths(
            node: &HashMap<String, IndexNode>,
            prefix: String,
            paths: &mut Vec<String>,
        ) {
            for (name, n) in node {
                match n {
                    IndexNode::File(entry) => paths.push(entry.path.clone()),
                    IndexNode::Directory(map) => {
                        let new_prefix = if prefix.is_empty() {
                            name.clone()
                        } else {
                            format!("{}/{}", prefix, name)
                        };
                        collect_paths(map, new_prefix, paths);
                    }
                }
            }
        }
        let mut paths = Vec::new();
        collect_paths(&self.entries, String::new(), &mut paths);
        paths
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}
