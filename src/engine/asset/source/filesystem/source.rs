use crate::engine::asset::source::priority::SourcePriority;
use crate::engine::asset::source::source::{AssetSource, SourceFileMetadata, SourceMetadata};
use std::path::Path;

/// 文件系统资源来源
///
/// 从本地文件系统读取资源。作为默认的最低优先级兜底来源。
pub struct FilesystemSource {
    root: String,
    enabled: bool,
}

impl FilesystemSource {
    /// 创建以指定目录为根的文件系统来源
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            enabled: true,
        }
    }

    /// 拼接完整路径
    fn full_path(&self, path: &str) -> String {
        format!("{}/{}", self.root, path)
    }
}

impl AssetSource for FilesystemSource {
    fn name(&self) -> &str {
        "Filesystem"
    }

    fn priority(&self) -> SourcePriority {
        SourcePriority::Filesystem
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(&self.full_path(path)).exists()
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(self.full_path(path)).map_err(|e| format!("fs read {}: {e}", path))
    }

    fn metadata(&self, path: &str) -> Option<SourceFileMetadata> {
        let meta = std::fs::metadata(self.full_path(path)).ok()?;
        Some(SourceFileMetadata {
            size: meta.len(),
            modified: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
            is_dir: meta.is_dir(),
            source_type: "filesystem".into(),
        })
    }

    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "Filesystem".into(),
            priority: self.priority(),
            version: 1,
            enabled: self.enabled,
            root_path: self.root.clone(),
            description: "Local filesystem asset source".into(),
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
