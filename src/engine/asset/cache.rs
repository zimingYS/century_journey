//! 缓存已解析资产，避免重复磁盘读取。

use bevy::prelude::*;
use std::collections::HashMap;

/// 泛型资源缓存 —— 用 Bevy `UntypedHandle` 统一存储不同类型资源的句柄。
/// 同一个 key（`AssetId::to_string()`）只加载一次，后续直接复用缓存的 Handle。
#[derive(Debug, Default)]
pub struct AssetCache {
    handles: HashMap<String, UntypedHandle>,
}

impl AssetCache {
    /// 返回指定键或索引对应的只读值。
    pub fn get<T: Asset>(&self, key: &str) -> Option<Handle<T>> {
        self.handles.get(key).map(|u| u.clone().typed::<T>())
    }

    /// 把指定值写入对应索引或缓存。
    pub fn insert<T: Asset>(&mut self, key: &str, handle: Handle<T>) {
        self.handles.insert(key.to_string(), handle.untyped());
    }

    /// 判断缓存中是否存在指定键。
    pub fn contains(&self, key: &str) -> bool {
        self.handles.contains_key(key)
    }

    /// 移除指定键对应的缓存条目。
    pub fn remove(&mut self, key: &str) {
        self.handles.remove(key);
    }

    /// 清空缓存中的全部资产句柄。
    pub fn clear(&mut self) {
        self.handles.clear();
    }

    /// 内部迭代——供 AssetManager::all_loaded() 聚合查询使用
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &UntypedHandle)> {
        self.handles.iter()
    }

    /// 返回当前集合中的条目数量。
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// 判断集合或缓存当前是否为空。
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}
