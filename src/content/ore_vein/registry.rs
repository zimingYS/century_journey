//! 把稳定矿脉定义解析为可供生成管线快速查询的运行时索引。

use crate::content::block::registry::BlockRegistry;
use crate::content::ore_vein::definition::OreVeinDefinition;
use crate::shared::identifier::Identifier;
use bevy::prelude::Resource;
use std::collections::HashSet;

/// 已解析方块运行时 ID 的矿脉记录。
#[derive(Debug, Clone)]
pub struct RuntimeOreVein {
    /// 矿脉的稳定内容定义。
    pub definition: OreVeinDefinition,
    /// 矿石方块在当前内容编译中的运行时 ID。
    pub block_id: u16,
}

/// 按优先级从高到低排序的只读矿脉注册表。
#[derive(Resource, Debug, Clone, Default)]
pub struct OreVeinRegistry {
    veins: Vec<RuntimeOreVein>,
}

impl OreVeinRegistry {
    /// 校验并原子替换全部矿脉定义；失败时保留原注册表。
    pub fn replace_definitions(
        &mut self,
        definitions: Vec<OreVeinDefinition>,
        block_registry: &BlockRegistry,
    ) -> Result<(), String> {
        let rebuilt = build_registry(definitions, |identifier| block_registry.get_id(identifier))?;
        *self = rebuilt;
        Ok(())
    }

    /// 按优先级从高到低遍历全部矿脉。
    pub fn iter(&self) -> impl Iterator<Item = &RuntimeOreVein> {
        self.veins.iter()
    }

    /// 返回当前矿脉数量。
    pub fn len(&self) -> usize {
        self.veins.len()
    }

    /// 判断当前是否没有可用矿脉。
    pub fn is_empty(&self) -> bool {
        self.veins.is_empty()
    }
}

fn build_registry(
    mut definitions: Vec<OreVeinDefinition>,
    mut resolve_block: impl FnMut(&Identifier) -> Option<u16>,
) -> Result<OreVeinRegistry, String> {
    // 优先级高者在前；同优先级按标识符排序，保证确定性。
    definitions.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.identifier.cmp(&right.identifier))
    });

    let mut identifiers = HashSet::new();
    let mut veins = Vec::with_capacity(definitions.len());
    for definition in definitions {
        if !identifiers.insert(definition.identifier.clone()) {
            return Err(format!(
                "duplicate ore vein identifier: {}",
                definition.identifier
            ));
        }
        let block_id = resolve_block(&definition.block).ok_or_else(|| {
            format!(
                "ore vein {} references unknown block {}",
                definition.identifier, definition.block
            )
        })?;
        if block_id == 0 {
            return Err(format!(
                "ore vein {} cannot use the air block",
                definition.identifier
            ));
        }
        veins.push(RuntimeOreVein {
            definition,
            block_id,
        });
    }

    Ok(OreVeinRegistry { veins })
}

#[cfg(test)]
#[path = "../../../tests/unit/content/ore_vein/registry.rs"]
mod tests;
