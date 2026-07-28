use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::{REGION_DIR_NAME, REGION_FILE_PREFIX};
use crate::game::save::config::SaveConfig;
use crate::game::save::world::format::{LevelData, SavedChunk};
use crate::game::save::world::level;
use crate::game::save::world::region::RegionManager;
use crate::game::world::state::WorldState;
use bevy::math::IVec3;
use bevy::prelude;
use bevy::prelude::{Res, ResMut, Resource};
use bincode::Options;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// 缓存的 block_id 重映射表
#[derive(Resource, Clone, Default)]
pub struct CachedBlockIdRemap(pub HashMap<u16, u16>);

/// 加载队列
#[derive(Resource, Default, Debug)]
pub struct LoadQueue {
    pub queue: VecDeque<SavedChunk>,
}

/// 从存档文件加载区块，加载到世界数据WorldState
pub fn process_load_queue_system(
    mut load_queue: ResMut<LoadQueue>,
    mut world_state: ResMut<WorldState>,
    save_config: Res<SaveConfig>,
    block_registry: Res<BlockRegistry>,
) {
    const MAX_LOAD_PER_FRAME: usize = 4;

    // 需要重映射的区块
    let level_data = level::load_level(&save_config.world_name).ok();
    let saved_id_map = level_data
        .as_ref()
        .map(|l| l.block_id_map.clone())
        .unwrap_or_default();

    let mut loaded = 0;
    while loaded < MAX_LOAD_PER_FRAME {
        let Some(saved) = load_queue.queue.pop_front() else {
            break;
        };

        let mut chunk_data = saved.data;

        // 如果存档中有 ID 映射，进行重映射
        if !saved_id_map.is_empty() {
            level::remap_chunk_block_ids(&mut chunk_data, &saved_id_map, &block_registry);
        }

        world_state.insert_chunk(saved.position, Arc::from(chunk_data));

        loaded += 1;
    }

    if loaded > 0 {
        log::trace!("[存档系统] 已加载 {} 个区块", loaded);
    }
}

/// 从存档文件读取单个区块
pub fn try_load_chunk_from_disk(world_name: &str, chunk_pos: IVec3) -> Option<SavedChunk> {
    match RegionManager::read_chunk(world_name, chunk_pos) {
        Ok(Some(saved)) => Some(saved),
        Ok(None) => None,
        Err(e) => {
            log::error!("[存档系统] 加载区块 {:?} 失败: {e}", chunk_pos);
            None
        }
    }
}

/// 加载整个世界
pub fn load_world_metadata(
    world_name: &str,
) -> prelude::Result<LevelData, super::region::SaveError> {
    level::load_level(world_name)
}

/// 从存档创建初始 WorldState
pub fn load_entire_world(
    world_name: &str,
    block_registry: &BlockRegistry,
) -> prelude::Result<(WorldState, LevelData), super::region::SaveError> {
    let level = level::load_level(world_name)?;
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
                                level::remap_chunk_block_ids(
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

/// 构建重映射表
pub fn cache_level_data_on_enter(
    save_config: Res<SaveConfig>,
    block_registry: Res<BlockRegistry>,
    mut cached_remap: ResMut<CachedBlockIdRemap>,
) {
    match level::load_level(&save_config.world_name) {
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
