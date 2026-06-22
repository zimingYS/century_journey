use bevy::prelude::*;
use std::collections::HashMap;

/// GLB/GLTF模型渲染器
/// 加载外部3D模型文件
pub struct HeldModelRenderer;

impl HeldModelRenderer {
    /// 加载模型 (占位 — 后续通过 AssetServer::load 实现)
    pub fn load_model(_asset_server: &AssetServer, _path: &str) {}
}

/// 模型 Mesh 缓存 — 避免重复加载
#[derive(Resource, Default)]
pub struct ModelMeshCache {
    pub scenes: HashMap<String, ()>,
}
