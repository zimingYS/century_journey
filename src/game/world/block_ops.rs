use crate::content::constant::world::CHUNK_SIZE;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;
use std::sync::Arc;

/// 根据世界坐标获取方块 ID
pub fn get_voxel_at_world(world_pos: IVec3, world_storage: &WorldStorage) -> u16 {
    let chunk_pos = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE as i32),
        world_pos.y.div_euclid(CHUNK_SIZE as i32),
        world_pos.z.div_euclid(CHUNK_SIZE as i32),
    );
    let local_x = world_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = world_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    world_storage
        .loaded_chunks
        .get(&chunk_pos)
        .map(|c| c.get_voxel(local_x, local_y, local_z))
        .unwrap_or(0)
}

/// 在世界坐标处设置方块 ID
pub fn set_voxel_at_world(world_pos: IVec3, block_id: u16, world_storage: &mut WorldStorage) {
    let chunk_pos = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE as i32),
        world_pos.y.div_euclid(CHUNK_SIZE as i32),
        world_pos.z.div_euclid(CHUNK_SIZE as i32),
    );
    let local_x = world_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = world_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    if let Some(arc) = world_storage.loaded_chunks.get_mut(&chunk_pos) {
        let chunk_data = Arc::make_mut(arc);
        chunk_data.set_voxel(local_x, local_y, local_z, block_id);
    }
}
