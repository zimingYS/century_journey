use crate::core::constant::CHUNK_SIZE;
use crate::player::systems::raycast::{raycast_voxel, TargetVoxel};
use crate::voxel::types::VoxelType;
use crate::world::chunk::{ChunkComponents, ChunkState};
use bevy::prelude::*;
use crate::world::storage::WorldStorage;

pub fn voxel_interaction_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    target_voxel: Res<TargetVoxel>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
){
    let left_click = mouse_button.just_pressed(MouseButton::Left);
    let right_click = mouse_button.just_pressed(MouseButton::Right);
    if !left_click && !right_click { return; }

    // 左键破坏，右键放置
    if let Some(ray_result) = &target_voxel.result {
        let (target_voxel, next_type) = if left_click {
            (ray_result.hit_pos, VoxelType::Air)
        } else {
            (ray_result.hit_pos + ray_result.normal, VoxelType::Grass)
        };

        // 获取击中的方块坐标
        let chunk_pos = IVec3::new(
            target_voxel.x.div_euclid(CHUNK_SIZE as i32),
            target_voxel.y.div_euclid(CHUNK_SIZE as i32),
            target_voxel.z.div_euclid(CHUNK_SIZE as i32),
        );

        // 换算局部坐标
        let local_x = target_voxel.x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = target_voxel.y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = target_voxel.z.rem_euclid(CHUNK_SIZE as i32) as usize;

        if let Some(chunk_data) = world_storage.loaded_chunks.get_mut(&chunk_pos) {
            chunk_data.set_voxel(local_x, local_y, local_z, next_type);

            let mut dirty_chunks = vec![chunk_pos];

            // 若修改的方块在区块边缘，把相邻区块也标记为脏
            if local_y == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, -1, 0)); }
            if local_y == 15 { dirty_chunks.push(chunk_pos + IVec3::new(0, 1, 0)); }
            if local_x == 0 { dirty_chunks.push(chunk_pos + IVec3::new(-1, 0, 0)); }
            if local_x == 15 { dirty_chunks.push(chunk_pos + IVec3::new(1, 0, 0)); }
            if local_z == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, -1)); }
            if local_z == 15 { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, 1)); }

            // 重新渲染当前区块
            for (chunk_comp, mut state) in &mut chunk_query {
                if dirty_chunks.contains(&chunk_comp.position) {
                    *state = ChunkState::DataReady;
                }
            }

            info!("成功修改方块坐标: {:?}, 类型变为: {:?}", target_voxel, next_type);
        }
    }
}