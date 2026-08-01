//! 把稳定树种定义解析为可供玩法快速查询的运行时索引。

use crate::content::block::registry::BlockRegistry;
use crate::content::vegetation::definition::TreeSpeciesDefinition;
use crate::shared::identifier::Identifier;
use bevy::prelude::Resource;
use std::collections::{HashMap, HashSet};

/// 已解析方块运行时 ID 的树种记录。
#[derive(Debug, Clone)]
pub struct RuntimeTreeSpecies {
    /// 树种的稳定内容定义。
    pub definition: TreeSpeciesDefinition,
    /// 树苗方块在当前内容编译中的运行时 ID。
    pub sapling_block_id: u16,
    /// 树干方块在当前内容编译中的运行时 ID。
    pub trunk_block_id: u16,
    /// 树叶方块在当前内容编译中的运行时 ID。
    pub leaves_block_id: u16,
}

/// 按树种标识和树苗方块 ID 提供双向查询的只读注册表。
#[derive(Resource, Debug, Clone, Default)]
pub struct TreeSpeciesRegistry {
    species: Vec<RuntimeTreeSpecies>,
    identifier_to_index: HashMap<Identifier, usize>,
    sapling_to_index: HashMap<u16, usize>,
}

impl TreeSpeciesRegistry {
    /// 校验并原子替换全部树种定义；失败时保留原注册表。
    pub fn replace_definitions(
        &mut self,
        definitions: Vec<TreeSpeciesDefinition>,
        block_registry: &BlockRegistry,
    ) -> Result<(), String> {
        let rebuilt = build_registry(definitions, |identifier| block_registry.get_id(identifier))?;
        *self = rebuilt;
        Ok(())
    }

    /// 按树种稳定标识符查询运行时记录。
    pub fn get(&self, identifier: &Identifier) -> Option<&RuntimeTreeSpecies> {
        self.identifier_to_index
            .get(identifier)
            .and_then(|&index| self.species.get(index))
    }

    /// 按世界中的树苗方块运行时 ID 查询树种。
    pub fn get_by_sapling_id(&self, block_id: u16) -> Option<&RuntimeTreeSpecies> {
        self.sapling_to_index
            .get(&block_id)
            .and_then(|&index| self.species.get(index))
    }

    /// 按稳定注册顺序遍历全部树种。
    pub fn iter(&self) -> impl Iterator<Item = &RuntimeTreeSpecies> {
        self.species.iter()
    }

    /// 返回当前树种数量。
    pub fn len(&self) -> usize {
        self.species.len()
    }

    /// 判断当前是否没有可用树种。
    pub fn is_empty(&self) -> bool {
        self.species.is_empty()
    }
}

fn build_registry(
    mut definitions: Vec<TreeSpeciesDefinition>,
    mut resolve_block: impl FnMut(&Identifier) -> Option<u16>,
) -> Result<TreeSpeciesRegistry, String> {
    definitions.sort_by(|left, right| left.identifier.cmp(&right.identifier));

    let mut identifiers = HashSet::new();
    let mut sapling_ids = HashSet::new();
    let mut species = Vec::with_capacity(definitions.len());
    for definition in definitions {
        if !identifiers.insert(definition.identifier.clone()) {
            return Err(format!(
                "duplicate tree species identifier: {}",
                definition.identifier
            ));
        }

        let sapling_block_id = resolve_block(&definition.sapling_block).ok_or_else(|| {
            format!(
                "tree species {} references unknown sapling block {}",
                definition.identifier, definition.sapling_block
            )
        })?;
        let trunk_block_id = resolve_block(&definition.trunk_block).ok_or_else(|| {
            format!(
                "tree species {} references unknown trunk block {}",
                definition.identifier, definition.trunk_block
            )
        })?;
        let leaves_block_id = resolve_block(&definition.leaves_block).ok_or_else(|| {
            format!(
                "tree species {} references unknown leaves block {}",
                definition.identifier, definition.leaves_block
            )
        })?;
        if [sapling_block_id, trunk_block_id, leaves_block_id].contains(&0) {
            return Err(format!(
                "tree species {} cannot use the air block",
                definition.identifier
            ));
        }
        if !sapling_ids.insert(sapling_block_id) {
            return Err(format!(
                "multiple tree species use sapling block {}",
                definition.sapling_block
            ));
        }

        species.push(RuntimeTreeSpecies {
            definition,
            sapling_block_id,
            trunk_block_id,
            leaves_block_id,
        });
    }

    let identifier_to_index = species
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry.definition.identifier.clone(), index))
        .collect();
    let sapling_to_index = species
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry.sapling_block_id, index))
        .collect();

    Ok(TreeSpeciesRegistry {
        species,
        identifier_to_index,
        sapling_to_index,
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/content/vegetation/registry.rs"]
mod tests;
