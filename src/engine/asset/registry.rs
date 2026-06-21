use std::collections::HashMap;
use bevy::prelude::*;
use crate::engine::asset::event::{AssetLoaded, AssetReloaded, AssetLoadFailed, AssetUnloaded};

/// 资源生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 加载失败
    Failed,
}

/// 资源注册表
#[derive(Resource, Debug, Default)]
pub struct AssetRegistry {
    states: HashMap<String, AssetState>,
}

impl AssetRegistry {
    /// 标记资源状态变更。
    pub fn set_state(&mut self, id: &str, state: AssetState) {
        self.states.insert(id.to_string(), state);
    }

    /// 查询资源状态。
    pub fn state(&self, id: &str) -> AssetState {
        self.states.get(id).copied().unwrap_or(AssetState::Unloaded)
    }

    /// 获取所有已加载的资源 ID。
    pub fn loaded_ids(&self) -> Vec<&String> {
        self.states
            .iter()
            .filter(|(_, s)| **s == AssetState::Loaded)
            .map(|(id, _)| id)
            .collect()
    }

    /// 获取所有加载失败的资源 ID。
    pub fn failed_ids(&self) -> Vec<&String> {
        self.states
            .iter()
            .filter(|(_, s)| **s == AssetState::Failed)
            .map(|(id, _)| id)
            .collect()
    }

    /// 发送加载完成事件。
    pub fn emit_loaded(&self, id: &str) -> AssetLoaded {
        AssetLoaded { id: id.to_string() }
    }

    /// 发送重新加载事件。
    pub fn emit_reloaded(&self, id: &str) -> AssetReloaded {
        AssetReloaded { id: id.to_string() }
    }

    /// 发送加载失败事件。
    pub fn emit_failed(&self, id: &str, error: &str) -> AssetLoadFailed {
        AssetLoadFailed {
            id: id.to_string(),
            error: error.to_string(),
        }
    }

    /// 发送卸载事件。
    pub fn emit_unloaded(&self, id: &str) -> AssetUnloaded {
        AssetUnloaded { id: id.to_string() }
    }
}
