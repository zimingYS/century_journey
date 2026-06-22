use crate::engine::asset::source::priority::SourcePriority;
use crate::engine::asset::source::source::{AssetSource, SourceFileMetadata, SourceMetadata};
use std::path::Path;

/// 资源包来源 — 从资源包目录加载资源
///
/// 每个资源包是一个包含 assets/ 子目录的独立目录。
/// 暂时仅支持目录形式（不做 ZIP 解包）。
pub struct ResourcePackSource {
    id: String,
    root_path: String,
    priority: SourcePriority,
    enabled: bool,
}

impl ResourcePackSource {
    pub fn new(
        id: impl Into<String>,
        root_path: impl Into<String>,
        priority: SourcePriority,
    ) -> Self {
        Self {
            id: id.into(),
            root_path: root_path.into(),
            priority,
            enabled: true,
        }
    }

    fn full_path(&self, path: &str) -> String {
        format!("{}/assets/{}", self.root_path, path)
    }
}

impl AssetSource for ResourcePackSource {
    fn name(&self) -> &str {
        &self.id
    }

    fn priority(&self) -> SourcePriority {
        self.priority
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(&self.full_path(path)).exists()
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(self.full_path(path))
            .map_err(|e| format!("resource_pack[{}] read {}: {e}", self.id, path))
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
            source_type: format!("resource_pack:{}", self.id),
        })
    }

    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: format!("ResourcePack:{}", self.id),
            priority: self.priority,
            version: 1,
            enabled: self.enabled,
            root_path: self.root_path.clone(),
            description: format!("Resource pack: {}", self.id),
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
