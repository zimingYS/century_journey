//! 从世界存档恢复元数据、区块数据和运行时方块 ID 映射。

use crate::content::block::registry::BlockRegistry;
use crate::game::save::SaveConfig;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::world::chunk::region::{RegionManager, SaveError};
use crate::game::save::world::metadata::io;
use crate::game::save::world::metadata::model::LevelData;
use crate::game::world::state::WorldState;
use bevy::prelude;
use bevy::prelude::{Res, ResMut, Resource};
use bincode::Options;
use std::collections::HashMap;
use std::sync::Arc;

/// 缓存存档方块 ID 到当前内容注册表 ID 的重映射关系。
#[derive(Resource, Clone, Default)]
pub struct CachedBlockIdRemap(pub HashMap<u16, u16>);

/// 进入世界时读取元数据并构建方块 ID 重映射表。
pub fn cache_level_data_on_enter(
    save_config: Res<SaveConfig>,
    block_registry: Res<BlockRegistry>,
    mut cached_remap: ResMut<CachedBlockIdRemap>,
) {
    match io::load_level(&save_config.world_name) {
        Ok(level_data) => {
            cached_remap.0 = block_registry.build_id_remap_table(&level_data.block_id_map);
            log::info!(
                "[存档系统] level.dat 已缓存，block_id_map 含 {} 条记录",
                level_data.block_id_map.len()
            );
        }
        Err(_) => {
            // 新存档没有 level.dat 属于正常情况，后续会使用纯生成模式。
            log::info!("[存档系统] 未找到 level.dat，将使用纯生成模式");
        }
    }
}

/// 从磁盘加载完整世界状态和对应的世界元数据。
pub fn load_entire_world(
    world_name: &str,
    block_registry: &BlockRegistry,
) -> prelude::Result<(WorldState, LevelData), SaveError> {
    let level = io::load_level(world_name)?;
    let mut storage = WorldState::default();

    let regions_dir = RegionManager::regions_path(world_name);
    if regions_dir.exists() {
        for entry in std::fs::read_dir(&regions_dir)? {
            let path = entry?.path();
            let Some(region_pos) = RegionManager::region_position_from_path(&path) else {
                continue;
            };

            let region_path = RegionManager::region_path(world_name, region_pos);
            if let Ok(region) = RegionManager::read_region_path(&region_path) {
                for compressed in &region.chunks {
                    if let Ok(decompressed) = RegionManager::decompress(compressed)
                        && let Ok(mut saved) = bincode::DefaultOptions::new()
                            .with_varint_encoding()
                            .deserialize::<SavedChunk>(&decompressed)
                    {
                        io::remap_chunk_block_ids(
                            &mut saved.data,
                            &level.block_id_map,
                            block_registry,
                        );
                        storage.insert_chunk(saved.position, Arc::from(saved.data));
                    }
                }
            }
        }
    }

    Ok((storage, level))
}
