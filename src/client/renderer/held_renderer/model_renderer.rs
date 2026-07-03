use crate::engine::asset::manager::AssetManager;
use bevy::prelude::*;
use std::collections::HashMap;

/// GLB/GLTF模型渲染器
pub struct HeldModelRenderer;

impl HeldModelRenderer {
    /// 加载模型 (通过 AssetManager)
    pub fn load_model(_asset: &mut AssetManager, _path: &str) {}
}

/// 模型 Mesh 缓存 — 避免重复加载
#[derive(Resource, Default)]
pub struct ModelMeshCache {
    pub scenes: HashMap<String, ()>,
}
