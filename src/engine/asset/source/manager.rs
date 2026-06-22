use crate::engine::asset::source::source::AssetSource;
use bevy::prelude::*;

/// 来源管理器 — 统一维护所有资源来源并按优先级查找
///
/// 业务代码和 Pipeline 不知道资源来自哪里。
/// SourceManager 自动按优先级遍历所有来源，返回第一个命中结果。
#[derive(Resource)]
pub struct SourceManager {
    sources: Vec<Box<dyn AssetSource>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// 注册一个来源
    pub fn add(&mut self, source: impl AssetSource) {
        self.sources.push(Box::new(source));
        // 按优先级排序（高优先级在前）
        self.sources.sort_by_key(|s| s.priority());
    }

    /// 注销指定名称的来源
    pub fn remove_by_name(&mut self, name: &str) {
        self.sources.retain(|s| s.name() != name);
    }

    /// 按优先级查找资源，返回第一个命中来源的数据 + 来源名称
    pub fn load_bytes(&self, path: &str) -> Result<(String, Vec<u8>), String> {
        for source in &self.sources {
            if source.is_enabled() && source.exists(path) {
                let bytes = source
                    .read(path)
                    .map_err(|e| format!("{}: {e}", source.name()))?;
                return Ok((source.name().to_string(), bytes));
            }
        }
        Err(format!("not found in any source: {path}"))
    }

    /// 检查资源是否在某个来源中存在
    pub fn exists(&self, path: &str) -> bool {
        self.sources
            .iter()
            .any(|s| s.is_enabled() && s.exists(path))
    }

    /// 获取来源数量
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// 遍历所有已启用的来源（只读）
    pub fn iter_enabled(&self) -> impl Iterator<Item = &dyn AssetSource> {
        self.sources
            .iter()
            .filter(|s| s.is_enabled())
            .map(|s| s.as_ref())
    }

    /// 禁用指定名称的来源
    pub fn disable(&mut self, name: &str) {
        for source in &mut self.sources {
            if source.name() == name {
                source.set_enabled(false);
            }
        }
    }

    /// 启用指定名称的来源
    pub fn enable(&mut self, name: &str) {
        for source in &mut self.sources {
            if source.name() == name {
                source.set_enabled(true);
            }
        }
    }
}

impl Default for SourceManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.add(super::filesystem::FilesystemSource::new("assets"));
        manager
    }
}
