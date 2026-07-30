//! 维护生物群系标识到定义的只读运行时索引。

use crate::content::biome::definition::BiomeDefinition;
use crate::shared::identifier::Identifier;
use bevy::prelude::Resource;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default, Clone)]
/// 按稳定索引保存已编译生物群系定义的注册表。
pub struct BiomeRegistry {
    biomes: Vec<BiomeDefinition>,
    identifier_to_index: HashMap<Identifier, u8>,
}

impl BiomeRegistry {
    /// 校验定义并创建稳定索引的生物群系注册表。
    pub fn from_definitions(definitions: Vec<BiomeDefinition>) -> Result<Self, String> {
        let mut registry = Self::default();
        registry.replace_definitions(definitions)?;
        Ok(registry)
    }

    /// 校验并原子替换全部生物群系定义。
    pub fn replace_definitions(
        &mut self,
        mut definitions: Vec<BiomeDefinition>,
    ) -> Result<(), String> {
        if definitions.is_empty() {
            return Err("at least one biome definition is required".into());
        }
        if definitions.len() > u8::MAX as usize + 1 {
            return Err(format!(
                "too many biome definitions: {}, maximum is 256",
                definitions.len()
            ));
        }
        definitions.sort_by(|left, right| {
            left.generation_order
                .cmp(&right.generation_order)
                .then_with(|| left.identifier.cmp(&right.identifier))
        });

        let mut identifiers = HashSet::new();
        let mut orders = HashSet::new();
        for biome in &definitions {
            if !identifiers.insert(biome.identifier.clone()) {
                return Err(format!("duplicate biome identifier: {}", biome.identifier));
            }
            if !orders.insert(biome.generation_order) {
                return Err(format!(
                    "duplicate biome generation_order: {}",
                    biome.generation_order
                ));
            }
        }

        self.biomes = definitions;
        self.identifier_to_index.clear();
        for (index, biome) in self.biomes.iter().enumerate() {
            self.identifier_to_index
                .insert(biome.identifier.clone(), index as u8);
        }
        Ok(())
    }

    /// 返回指定键或索引对应的只读值。
    pub fn get(&self, index: u8) -> Option<&BiomeDefinition> {
        self.biomes.get(index as usize)
    }

    /// 按稳定注册顺序遍历所有生物群系。
    pub fn biomes_iter(&self) -> impl Iterator<Item = (usize, &BiomeDefinition)> {
        self.biomes.iter().enumerate()
    }

    /// 返回当前集合中的条目数量。
    pub fn len(&self) -> usize {
        self.biomes.len()
    }

    /// 判断集合或缓存当前是否为空。
    pub fn is_empty(&self) -> bool {
        self.biomes.is_empty()
    }

    /// 返回指定生物群系标识的稳定索引。
    pub fn get_index(&self, identifier: &str) -> Option<u8> {
        let key = Identifier::parse(identifier).ok()?;
        self.identifier_to_index.get(&key).copied()
    }
}
