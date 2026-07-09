use std::collections::HashMap;

use bevy::prelude::*;

use crate::shared::identifier::Identifier;

use super::baked_model::BakedItemModel;

/// 物品模型运行时缓存。
///
/// content 层的 ItemModelRegistry 负责保存可序列化定义；这里仅缓存 Bevy 运行时资源，避免重复烘焙 mesh 和 material。
#[derive(Resource, Default)]
pub struct ItemModelCache {
    /// 模型 ID -> 已烘焙好的物品模型。
    baked_models: HashMap<Identifier, BakedItemModel>,
    /// mesh 构建键 -> Bevy mesh 句柄。
    meshes: HashMap<String, Handle<Mesh>>,
}

impl ItemModelCache {
    /// 查询已经烘焙好的物品模型。
    pub fn get_model(&self, identifier: &Identifier) -> Option<&BakedItemModel> {
        self.baked_models.get(identifier)
    }

    /// 写入已经烘焙好的物品模型。
    pub fn insert_model(&mut self, identifier: Identifier, model: BakedItemModel) {
        self.baked_models.insert(identifier, model);
    }

    /// 查询可复用 mesh。
    pub fn mesh(&self, key: &str) -> Option<&Handle<Mesh>> {
        self.meshes.get(key)
    }

    /// 写入可复用 mesh。
    pub fn insert_mesh(&mut self, key: impl Into<String>, mesh: Handle<Mesh>) {
        self.meshes.insert(key.into(), mesh);
    }

    /// 清空所有运行时缓存。
    ///
    /// 当物品定义、模型定义或贴图资源被重新加载时，外层系统可以调用它强制下一帧重新烘焙。
    pub fn clear(&mut self) {
        self.baked_models.clear();
        self.meshes.clear();
    }
}
