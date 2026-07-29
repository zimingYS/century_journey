use crate::content::block::registry::BlockRegistry;
use crate::game::save::config::SaveConfig;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::world::chunk::region::RegionManager;
use crate::game::save::world::metadata::io;
use crate::game::world::state::authoritative::WorldState;
use bevy::math::IVec3;
use bevy::prelude::{Res, ResMut, Resource};
use std::collections::VecDeque;
use std::sync::Arc;

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
    let level_data = io::load_level(&save_config.world_name).ok();
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
            io::remap_chunk_block_ids(&mut chunk_data, &saved_id_map, &block_registry);
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
