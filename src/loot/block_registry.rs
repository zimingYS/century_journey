use std::collections::HashMap;
use bevy::prelude::*;
use crate::inventory::item::id::ItemId;
use crate::inventory::item::stack::ItemStack;
use crate::loot::table::{LootEntry, LootTable};
use crate::voxel::registry::BlockRegistry;

/// 方块掉落注册表（BlockId → LootTable）
#[derive(Resource, Default)]
pub struct BlockLootRegistry {
    tables: HashMap<u16, LootTable>,
}

impl BlockLootRegistry {
    /// 注册方块掉落
    pub fn register(&mut self, block_id: u16, table: LootTable) {
        self.tables.insert(block_id, table);
    }

    /// 查询方块掉落
    pub fn get(&self, block_id: u16) -> Option<&LootTable> {
        self.tables.get(&block_id)
    }

    /// 计算方块的掉落物品列表
    pub fn roll(&self, block_id: u16) -> Vec<ItemStack> {
        self.tables
            .get(&block_id)
            .map(|t| t.roll())
            .unwrap_or_default()
    }
}

/// 初始化默认方块掉落表
pub fn init_default_loot_system(
    block_registry: Res<BlockRegistry>,
    mut loot_registry: ResMut<BlockLootRegistry>,
) {
    for (&block_id, identifier) in &block_registry.id_to_identifier {
        if block_id == 0 {
            continue; // 空气不掉落
        }

        let item_id = ItemId::block(identifier.as_str());

        // 特殊覆盖
        let table = match identifier.as_str() {
            "century_journey:grass" | "century_journey:grass_block" => {
                LootTable::single(ItemId::block("century_journey:dirt"), 1)
            }
            "century_journey:leaves" | "century_journey:oak_leaves" => {
                LootTable::single(ItemId::block("century_journey:stick"), 1)
                    .with(LootEntry {
                        item: ItemId::block("century_journey:oak_sapling"),
                        min_count: 1, max_count: 1, chance: 0.05,
                    })
            }
            _ => {
                // 默认：掉落自身
                LootTable::single(item_id, 1)
            }
        };

        loot_registry.register(block_id, table);
    }

    info!("[BlockLootRegistry] 已注册 {} 个方块掉落表", loot_registry.tables.len());
}