use crate::engine::asset::source::memory::storage::MemoryStorage;
use crate::engine::asset::source::priority::SourcePriority;
use crate::engine::asset::source::source::{AssetSource, SourceFileMetadata, SourceMetadata};

/// 内存资源来源
///
/// 资源保存在内存中。主要用于 Editor / Runtime Generated / Unit Test。
/// 支持动态注册和动态删除。
pub struct MemorySource {
    storage: MemoryStorage,
    name: String,
    enabled: bool,
}

impl MemorySource {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            storage: MemoryStorage::new(),
            name: name.into(),
            enabled: true,
        }
    }

    /// 动态注册资源
    pub fn insert(&mut self, path: &str, bytes: Vec<u8>) {
        self.storage.insert(path, bytes);
    }

    /// 动态删除资源
    pub fn remove(&mut self, path: &str) {
        self.storage.remove(path);
    }

    /// 清空内存
    pub fn clear(&mut self) {
        self.storage.clear();
    }

    /// 存储条目数
    pub fn len(&self) -> usize {
        self.storage.len()
    }
}

impl AssetSource for MemorySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> SourcePriority {
        SourcePriority::Memory
    }

    fn exists(&self, path: &str) -> bool {
        self.storage.contains(path)
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        self.storage
            .get(path)
            .cloned()
            .ok_or_else(|| format!("memory read: {} not found", path))
    }

    fn metadata(&self, path: &str) -> Option<SourceFileMetadata> {
        self.storage.get(path).map(|bytes| SourceFileMetadata {
            size: bytes.len() as u64,
            modified: None,
            is_dir: false,
            source_type: "memory".into(),
        })
    }

    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: self.name.clone(),
            priority: self.priority(),
            version: 1,
            enabled: self.enabled,
            root_path: "(memory)".into(),
            description: "In-memory asset source".into(),
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
