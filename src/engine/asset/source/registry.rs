use crate::engine::asset::source::source::SourceMetadata;
use bevy::prelude::*;

/// 来源注册表 — 记录所有已注册的 Source
///
/// 维护每个 Source 的类型、优先级、状态、是否启用。
/// 不与 SourceManager 的优先级查找冲突（Registry 用于调试/UI，Manager 用于运行时查找）。
#[derive(Resource, Default)]
pub struct SourceRegistry {
    entries: Vec<SourceMetadata>,
}

impl SourceRegistry {
    /// 注册一个新的来源
    pub fn register(&mut self, meta: SourceMetadata) {
        self.entries.push(meta);
    }

    /// 获取所有来源
    pub fn all(&self) -> &[SourceMetadata] {
        &self.entries
    }

    /// 按优先级排序并返回
    pub fn sorted_by_priority(&self) -> Vec<&SourceMetadata> {
        let mut sorted: Vec<&SourceMetadata> = self.entries.iter().collect();
        sorted.sort_by_key(|m| m.priority);
        sorted
    }

    /// 获取启用的来源
    pub fn enabled(&self) -> Vec<&SourceMetadata> {
        self.entries.iter().filter(|m| m.enabled).collect()
    }

    /// 按名称查找
    pub fn find_by_name(&self, name: &str) -> Option<&SourceMetadata> {
        self.entries.iter().find(|m| m.name == name)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
