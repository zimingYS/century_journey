use bevy::prelude::*;
use std::collections::HashMap;

/// 手持物品 Mesh 缓存
#[derive(Resource, Default)]
pub struct HeldMeshCache {
    pub meshes: HashMap<String, Handle<Mesh>>,
}

impl HeldMeshCache {
    pub fn get(&self, key: &str) -> Option<&Handle<Mesh>> {
        self.meshes.get(key)
    }

    pub fn insert(&mut self, key: String, handle: Handle<Mesh>) {
        self.meshes.insert(key, handle);
    }
}
