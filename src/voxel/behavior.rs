use std::sync::Arc;
use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::voxel::registry::BlockRegistry;
use crate::world::storage::WorldStorage;

/// 定义方块对各种事件的响应
pub trait BlockBehavior: Send + Sync + 'static {
    /// 方块被破坏
    fn on_break(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _world_storage: &mut WorldStorage,
        _commands: &mut Commands,
    ) {}

    /// 方块被放置时调用，返回 false 可取消放置
    fn on_place(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _face_normal: IVec3,
        _world_storage: &mut WorldStorage,
        _commands: &mut Commands,
    ) -> bool {
        true
    }

    /// 方块被右键交互时调用
    fn on_interact(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _face_normal: IVec3,
        _interactor: Option<Entity>,
        _world_storage: &mut WorldStorage,
        _commands: &mut Commands,
    ) {}

    /// 方块状态更新
    fn on_change(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _world_storage: &mut WorldStorage,
        _commands: &mut Commands,
    ) {}

    /// 邻居方块变更时调用
    fn on_neighbor_update(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _neighbor_pos: IVec3,
        _neighbor_block_id: u16,
        _world_storage: &mut WorldStorage,
        _commands: &mut Commands,
    ) {}
}

/// 默认空行为
pub struct DefaultBlockBehavior;
impl BlockBehavior for DefaultBlockBehavior {}

// 其他行为举例实现
/*
    /// 沙子/砾石方块行为 — 受重力影响
    pub struct FallingBlockBehavior;

    impl BlockBehavior for FallingBlockBehavior {
        fn on_place(){}
    }
*/

// 辅助函数
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

pub fn is_solid(block_id: u16, block_registry: &BlockRegistry) -> bool {
    block_registry
        .get(block_id)
        .map(|p| p.is_solid)
        .unwrap_or(false)
}
