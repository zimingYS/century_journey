use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::{REGION_DIR_NAME, REGION_FILE_PREFIX};
use crate::game::save::SaveConfig;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::world::chunk::region::{RegionManager, SaveError};
use crate::game::save::world::metadata::io;
use crate::game::save::world::metadata::model::LevelData;
use crate::game::world::state::WorldState;
use bevy::math::IVec3;
use bevy::prelude;
use bevy::prelude::{Res, ResMut, Resource};
use bincode::Options;
use std::collections::HashMap;
use std::sync::Arc;

/// 缓存的 block_id 重映射表
#[derive(Resource, Clone, Default)]
pub struct CachedBlockIdRemap(pub HashMap<u16, u16>);

/// 构建重映射表
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
            // 新存档没有 level.dat，正常
            log::info!("[存档系统] 未找到 level.dat，将使用纯生成模式");
        }
    }
}

/// 从存档创建初始 WorldState
pub fn load_entire_world(
    world_name: &str,
    block_registry: &BlockRegistry,
) -> prelude::Result<(WorldState, LevelData), SaveError> {
    let level = io::load_level(world_name)?;
    let mut storage = WorldState::default();

    // 遍历所有 region 文件
    let regions_dir = RegionManager::save_root(world_name).join(REGION_DIR_NAME);
    if regions_dir.exists() {
        for entry in std::fs::read_dir(&regions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "bin") {
                // 解析文件名获取 region 坐标
                let stem = path.file_stem().unwrap().to_string_lossy();
                let parts: Vec<&str> = stem.split('.').collect();
                if parts.len() == 4
                    && parts[0] == REGION_FILE_PREFIX
                    && let (Ok(rx), Ok(ry), Ok(rz)) = (
                        parts[1].parse::<i32>(),
                        parts[2].parse::<i32>(),
                        parts[3].parse::<i32>(),
                    )
                {
                    let region_pos = IVec3::new(rx, ry, rz);

                    // 读取该 region 中所有区块
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
        }
    }

    Ok((storage, level))
}
