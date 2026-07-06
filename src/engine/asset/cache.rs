use bevy::prelude::*;
use std::collections::HashMap;

/// 泛型资源缓存 —— 用 Bevy `UntypedHandle` 统一存储不同类型资源的句柄。
/// 同一个 key（`AssetId::to_string()`）只加载一次，后续直接复用缓存的 Handle。
#[derive(Debug, Default)]
pub struct AssetCache {
    handles: HashMap<String, UntypedHandle>,
}

impl AssetCache {
    pub fn get<T: Asset>(&self, key: &str) -> Option<Handle<T>> {
        self.handles.get(key).map(|u| u.clone().typed::<T>())
    }

    pub fn insert<T: Asset>(&mut self, key: &str, handle: Handle<T>) {
        self.handles.insert(key.to_string(), handle.untyped());
    }

    pub fn contains(&self, key: &str) -> bool {
        self.handles.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.handles.remove(key);
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}
