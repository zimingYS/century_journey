use std::collections::HashMap;

use bevy::prelude::*;

use crate::shared::identifier::Identifier;

use super::definition::ItemModelDefinition;

/// 物品模型定义注册表。
///
/// 只保存可序列化的模型定义，不保存 Mesh / Material。运行时资源由客户端 ItemModelCache 管理。
#[derive(Resource, Default)]
pub struct ItemModelRegistry {
    /// 模型 ID -> 模型定义。
    models: HashMap<Identifier, ItemModelDefinition>,
}

impl ItemModelRegistry {
    /// 注册模型定义，并把 identifier 回写到定义内。
    pub fn register(
        &mut self,
        identifier: Identifier,
        mut model: ItemModelDefinition,
    ) -> Identifier {
        model.identifier = Some(identifier.clone());
        self.models.insert(identifier.clone(), model);
        identifier
    }

    /// 查询模型定义。
    pub fn get(&self, identifier: &Identifier) -> Option<&ItemModelDefinition> {
        self.models.get(identifier)
    }

    /// 判断模型是否存在。
    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.models.contains_key(identifier)
    }

    /// 遍历所有模型定义。
    pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &ItemModelDefinition)> {
        self.models.iter()
    }

    /// 返回模型数量。
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// 判断注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}
