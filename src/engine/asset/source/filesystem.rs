use std::fs;
use std::path::Path;
use crate::engine::asset::source::AssetSource;

/// 文件系统资源来源。
///
/// 从本地文件系统读取资源数据。
pub struct FileSystemSource {
    root: String,
}

impl FileSystemSource {
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into() }
    }
}

impl AssetSource for FileSystemSource {
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, String> {
        let full = Path::new(&self.root).join(path);
        fs::read(&full).map_err(|e| format!("Failed to read {}: {}", full.display(), e))
    }
}
