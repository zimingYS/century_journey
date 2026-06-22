use crate::engine::asset::source::priority::SourcePriority;
use crate::engine::asset::source::source::{AssetSource, SourceFileMetadata, SourceMetadata};
use std::collections::HashMap;

/// 内存字节存储
#[derive(Default)]
pub struct MemoryStorage {
    entries: HashMap<String, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, path: &str, bytes: Vec<u8>) {
        self.entries.insert(path.to_string(), bytes);
    }

    pub fn remove(&mut self, path: &str) {
        self.entries.remove(path);
    }

    pub fn get(&self, path: &str) -> Option<&Vec<u8>> {
        self.entries.get(path)
    }

    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
