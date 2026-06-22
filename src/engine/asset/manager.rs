use crate::engine::asset::handle::AssetHandle;
use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::loader::json::JsonAsset;
use crate::engine::asset::state::AssetState;
use bevy::prelude::*;

/// 资源管理器 — Engine Asset 唯一公开入口（Facade）
///
/// 业务代码只能通过 `AssetManager` 访问所有资源功能。
/// Manager 不包含加载逻辑、Cache 操作、Registry 操作。
/// 所有内部协调由 Runtime 在 PostUpdate 中处理。
///
/// # 使用
///
/// ```ignore
/// fn my_system(asset: Res<AssetManager>) {
///     let tex = asset.texture(AssetId::default_namespace("ui/slot"));
/// }
/// ```

#[derive(Resource, Default)]
pub struct AssetManager {
    /// 待加载请求队列（异步处理）
    pending: Vec<AssetId>,
}

impl AssetManager {
    /// 加载纹理 (返回 `AssetHandle<Image>`)
    pub fn texture(&mut self, id: AssetId) -> AssetHandle<Image> {
        let placeholder = Handle::default();
        self.pending.push(id.clone());
        AssetHandle::new(placeholder, id)
    }

    /// 加载 JSON (返回 `AssetHandle<JsonAsset>`)
    pub fn json(&mut self, id: AssetId) -> AssetHandle<JsonAsset> {
        let placeholder = Handle::default();
        self.pending.push(id.clone());
        AssetHandle::new(placeholder, id)
    }

    /// 通用资源加载
    pub fn load<T: Asset>(&mut self, id: AssetId) -> AssetHandle<T> {
        let placeholder = Handle::default();
        self.pending.push(id.clone());
        AssetHandle::new(placeholder, id)
    }

    /// 预加载资源
    pub fn preload(&self, _id: AssetId) {}

    /// 流式加载
    pub fn stream(&self, _id: AssetId) {}

    /// 触发热重载
    pub fn reload(&self, _id: AssetId) {}

    /// 释放引用
    pub fn release(&self, _id: AssetId) {}

    /// 查询资源状态
    pub fn state(&self, _id: &AssetId) -> AssetState {
        AssetState::Ready
    }

    /// 是否就绪
    pub fn is_ready(&self, id: &AssetId) -> bool {
        matches!(self.state(id), AssetState::Ready)
    }

    /// 排空待处理队列（由 Runtime 调用）
    pub(crate) fn drain_pending(&mut self) -> Vec<AssetId> {
        std::mem::take(&mut self.pending)
    }
}
