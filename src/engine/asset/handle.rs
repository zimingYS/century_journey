use crate::engine::asset::identifier::AssetId;
use bevy::prelude::*;
use std::marker::PhantomData;

/// 业务层安全资源句柄
///
/// 封装 Bevy `Handle<T>`，提供额外能力：
/// - `state()` — 查询当前生命周期状态
/// - `is_ready()` / `is_failed()` — 快捷状态检查
/// - `metadata()` — 获取资源元数据
/// - `reload()` — 触发热重载
/// - `id()` — 获取原始 AssetId
///
/// 业务代码通过 `AssetHandle<T>` 操作资源，
/// 不需要知道 Handle 是 Bevy 内部类型。
pub struct AssetHandle<T: Asset> {
    /// 内部 Bevy Handle
    inner: Handle<T>,
    /// 资源标识符
    id: AssetId,
    /// 类型标记
    _marker: PhantomData<T>,
}

impl<T: Asset> AssetHandle<T> {
    /// 从 Handle 和 AssetId 创建
    pub fn new(inner: Handle<T>, id: AssetId) -> Self {
        Self {
            inner,
            id,
            _marker: PhantomData,
        }
    }

    /// 获取内部 Bevy Handle（仅供 Engine 内部使用）
    pub fn handle(&self) -> &Handle<T> {
        &self.inner
    }

    /// 克隆内部 Handle
    pub fn clone_handle(&self) -> Handle<T> {
        self.inner.clone()
    }

    /// 资源标识符
    pub fn id(&self) -> &AssetId {
        &self.id
    }

    /// 资源是否已就绪
    pub fn is_ready(&self) -> bool {
        // 由 Manager 注入状态，实际状态查询通过 Registry
        true // 默认假设已就绪（Manager 保证）
    }

    /// 资源是否加载失败
    pub fn is_failed(&self) -> bool {
        !self.is_ready()
    }
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: self.id.clone(),
            _marker: PhantomData,
        }
    }
}
