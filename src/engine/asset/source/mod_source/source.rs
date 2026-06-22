use crate::engine::asset::source::priority::SourcePriority;
use crate::engine::asset::source::source::{AssetSource, SourceFileMetadata, SourceMetadata};
use std::path::Path;

/// Mod 资源来源
///
/// Mod 拥有独立的 namespace 和资源目录。
/// 资源覆盖通过优先级实现（Mod > ResourcePack > Filesystem）。
pub struct ModSource {
    /// Mod 唯一标识符
    mod_id: String,
    /// 命名空间
    namespace: String,
    /// Mod 根目录
    root_path: String,
    /// 是否启用
    enabled: bool,
}

impl ModSource {
    pub fn new(
        mod_id: impl Into<String>,
        namespace: impl Into<String>,
        root_path: impl Into<String>,
    ) -> Self {
        Self {
            mod_id: mod_id.into(),
            namespace: namespace.into(),
            root_path: root_path.into(),
            enabled: true,
        }
    }

    fn full_path(&self, path: &str) -> String {
        format!("{}/assets/{}", self.root_path, path)
    }

    /// 是否为该 Mod 负责的命名空间
    pub fn matches_namespace(&self, ns: &str) -> bool {
        self.namespace == ns
    }
}

impl AssetSource for ModSource {
    fn name(&self) -> &str {
        &self.mod_id
    }

    fn priority(&self) -> SourcePriority {
        SourcePriority::Mod
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(&self.full_path(path)).exists()
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(self.full_path(path))
            .map_err(|e| format!("mod[{}] read {}: {e}", self.mod_id, path))
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
            source_type: format!("mod:{}", self.namespace),
        })
    }

    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: format!("Mod:{}", self.mod_id),
            priority: self.priority(),
            version: 1,
            enabled: self.enabled,
            root_path: self.root_path.clone(),
            description: format!("Mod: {} (namespace: {})", self.mod_id, self.namespace),
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
