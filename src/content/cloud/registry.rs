//! 维护编译后云定义的只读注册表。

use crate::content::cloud::definition::CloudDefinition;
use crate::shared::identifier::Identifier;
use bevy::prelude::Resource;
use std::collections::HashMap;

/// 编译并校验后的云定义注册表。
///
/// 注册表按稳定标识符索引云场，渲染层只读取定义，不持有任何实体或 GPU 资源。
#[derive(Resource, Debug, Clone, Default)]
pub struct CloudRegistry {
    definitions: Vec<CloudDefinition>,
    identifier_to_index: HashMap<Identifier, usize>,
}

impl CloudRegistry {
    /// 使用编译结果原子替换全部云定义，并建立稳定标识索引。
    pub fn replace_definitions(&mut self, mut definitions: Vec<CloudDefinition>) {
        definitions.sort_by(|left, right| left.identifier.cmp(&right.identifier));
        let identifier_to_index = definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| (definition.identifier.clone(), index))
            .collect();
        self.definitions = definitions;
        self.identifier_to_index = identifier_to_index;
    }

    /// 返回当前云定义数量。
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// 判断注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// 按稳定标识符查询云场定义。
    pub fn get(&self, identifier: &Identifier) -> Option<&CloudDefinition> {
        self.identifier_to_index
            .get(identifier)
            .and_then(|&index| self.definitions.get(index))
    }

    /// 返回稳定排序后的第一份云定义，作为尚未选择具体云场时的默认配置。
    pub fn primary(&self) -> Option<&CloudDefinition> {
        self.definitions.first()
    }
}
