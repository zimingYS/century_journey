use crate::engine::asset::event::{
    AssetChanged, AssetLoadFailed, AssetLoaded, AssetRegistered, AssetReloaded, AssetUnloaded,
};
use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::state::{AssetMetadata, AssetState};
use bevy::prelude::*;
use std::collections::HashMap;

/// 资源注册表
///
/// 负责跟踪所有已注册资源的元数据和生命周期状态。
/// 不保存 Handle（Handle 由 `AssetCache` 管理）。
#[derive(Resource, Debug, Default)]
pub struct AssetRegistry {
    entries: HashMap<String, AssetMetadata>,
}

impl AssetRegistry {
    /// 注册新资源
    pub fn register(&mut self, id: AssetId, source: impl Into<String>) -> AssetRegistered {
        let key = id.to_string();
        let meta = AssetMetadata::new(id, source);
        self.entries.insert(key.clone(), meta);
        AssetRegistered { id: key }
    }

    /// 设置资源状态
    pub fn set_state(&mut self, key: &str, state: AssetState) {
        if let Some(meta) = self.entries.get_mut(key) {
            meta.state = state;
            if matches!(state, AssetState::Ready) && meta.load_time.is_none() {
                meta.load_time = Some(now_secs());
            }
        }
    }

    /// 获取状态
    pub fn state(&self, key: &str) -> AssetState {
        self.entries
            .get(key)
            .map(|m| m.state)
            .unwrap_or(AssetState::Disposed)
    }

    /// 获取元数据
    pub fn metadata(&self, key: &str) -> Option<&AssetMetadata> {
        self.entries.get(key)
    }

    /// 增加引用计数
    pub fn increment_ref(&mut self, key: &str) {
        if let Some(meta) = self.entries.get_mut(key) {
            meta.ref_count += 1;
        }
    }

    /// 减少引用计数
    pub fn decrement_ref(&mut self, key: &str) {
        if let Some(meta) = self.entries.get_mut(key) {
            meta.ref_count = meta.ref_count.saturating_sub(1);
        }
    }

    /// 记录错误
    pub fn set_error(&mut self, key: &str, error: impl Into<String>) {
        if let Some(meta) = self.entries.get_mut(key) {
            meta.state = AssetState::Failed;
            meta.last_error = Some(error.into());
        }
    }

    /// Ready 状态的资源列表
    pub fn ready_ids(&self) -> Vec<&String> {
        self.entries
            .iter()
            .filter(|(_, m)| m.state == AssetState::Ready)
            .map(|(k, _)| k)
            .collect()
    }

    /// Failed 状态列表
    pub fn failed_ids(&self) -> Vec<&String> {
        self.entries
            .iter()
            .filter(|(_, m)| m.state == AssetState::Failed)
            .map(|(k, _)| k)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    // —— 事件 ——

    pub fn emit_loaded(&self, key: &str) -> AssetLoaded {
        AssetLoaded {
            id: key.to_string(),
        }
    }

    pub fn emit_reloaded(&self, key: &str) -> AssetReloaded {
        AssetReloaded {
            id: key.to_string(),
        }
    }

    pub fn emit_failed(&self, key: &str, error: &str) -> AssetLoadFailed {
        AssetLoadFailed {
            id: key.to_string(),
            error: error.to_string(),
        }
    }

    pub fn emit_unloaded(&self, key: &str) -> AssetUnloaded {
        AssetUnloaded {
            id: key.to_string(),
        }
    }

    pub fn emit_changed(&self, key: &str) -> AssetChanged {
        AssetChanged {
            id: key.to_string(),
        }
    }
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
