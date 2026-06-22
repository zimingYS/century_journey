use crate::engine::asset::identifier::AssetId;
use bevy::prelude::*;
use std::collections::HashMap;

/// Runtime 上下文 — 所有 Service 共享的数据层
///
/// Service 之间禁止直接互相引用，全部通过 Context 访问数据。
#[derive(Resource, Default)]
pub struct RuntimeContext {
    /// 引用计数：key → strong+weak counts
    pub references: HashMap<String, ReferenceEntry>,
    /// 资源信息：key → 元数据快照
    pub assets: HashMap<String, AssetInfo>,
    /// 运行时诊断
    pub diagnostics: DiagnosticsSnapshot,
}

/// 引用条目（增强版）
#[derive(Debug, Clone, Default)]
pub struct ReferenceEntry {
    /// 强引用计数（活跃使用中）
    pub strong_count: u32,
    /// 弱引用计数（缓存/可释放）
    pub weak_count: u32,
    /// 最后访问时间戳
    pub last_access: f64,
    /// 首次加载时间戳
    pub first_loaded: f64,
    /// 最后重载时间戳
    pub last_reload: f64,
    /// 拥有者标签（如 "chunk", "ui", "player"）
    pub owner: String,
}

/// 资源运行时信息
#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub id: AssetId,
    pub asset_type: String,
    pub size_bytes: u64,
    pub state: String,
    pub source: String,
    pub dependencies: Vec<AssetId>,
    pub version: u32,
    pub hash: u64,
}

/// 诊断快照
#[derive(Debug, Clone, Default)]
pub struct DiagnosticsSnapshot {
    pub loaded_count: u32,
    pub failed_count: u32,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_load_time_ms: f64,
    pub avg_load_time_ms: f64,
    pub reload_count: u32,
    pub streamed_count: u32,
    pub auto_unload_count: u32,
    pub memory_usage_bytes: u64,
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// 增加强引用
    pub fn retain(&mut self, key: &str) {
        let entry = self.references.entry(key.to_string()).or_default();
        entry.strong_count += 1;
        entry.last_access = now_secs();
    }

    /// 减少强引用
    pub fn release(&mut self, key: &str) {
        if let Some(entry) = self.references.get_mut(key) {
            entry.strong_count = entry.strong_count.saturating_sub(1);
        }
    }

    /// 是否未使用
    pub fn is_unused(&self, key: &str) -> bool {
        self.references
            .get(key)
            .map_or(true, |e| e.strong_count == 0)
    }

    /// 按最后访问时间排序返回未使用条目
    pub fn lru_unused(&self) -> Vec<String> {
        let mut items: Vec<_> = self
            .references
            .iter()
            .filter(|(_, e)| e.strong_count == 0)
            .collect();
        items.sort_by(|a, b| {
            a.1.last_access
                .partial_cmp(&b.1.last_access)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        items.into_iter().map(|(k, _)| k.clone()).collect()
    }
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
