use crate::player::systems::raycast::{raycast_voxel, TargetVoxel};
use crate::world::chunk::{ChunkComponents, ChunkState};
use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::core::input_block::InputBlocked;
use crate::ui::resources::inventory_ui_state::InventoryUiState;
use crate::voxel::registry::BlockRegistry;
use crate::world::storage::WorldStorage;

pub fn voxel_interaction_system(
    target_voxel: Res<TargetVoxel>,
    registry: Option<Res<BlockRegistry>>,
    input_blocked: Res<InputBlocked>,
    inventory_ui_state: Res<InventoryUiState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
){
    let Some(reg) = registry else { return; };
    // 当打开物品栏时不进行破坏和放置操作
    if input_blocked.0 { return; }

    let left_click = mouse_button.just_pressed(MouseButton::Left);
    let right_click = mouse_button.just_pressed(MouseButton::Right);
    if !left_click && !right_click { return; }

    // 左键破坏，右键放置
    if let Some(ray_result) = &target_voxel.result {
        let (target_pos, next_voxel_id) = if left_click {
            (ray_result.hit_pos, 0u16)
        } else {
            // 右键放置：从快捷栏里拿出当前手持方块的名称串唯一标识
            let current_hand_identifier = &inventory_ui_state.hotbar_items[inventory_ui_state.active_hotbar_index];

            // 翻译成运行时对应的动态ID
            let Some(block_id) = reg.get_id_by_identifier(current_hand_identifier) else { return; };
            if block_id == 0 { return; }

            (ray_result.hit_pos + ray_result.normal, block_id)
        };

        // 获取击中的方块坐标
        let chunk_pos = IVec3::new(
            target_pos.x.div_euclid(CHUNK_SIZE as i32),
            target_pos.y.div_euclid(CHUNK_SIZE as i32),
            target_pos.z.div_euclid(CHUNK_SIZE as i32),
        );

        // 换算局部坐标
        let local_x = target_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = target_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = target_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

        if let Some(chunk_data) = world_storage.loaded_chunks.get_mut(&chunk_pos) {
            chunk_data.set_voxel(local_x, local_y, local_z, next_voxel_id);

            let mut dirty_chunks = vec![chunk_pos];
            let max_idx = CHUNK_SIZE - 1;

            // 若修改的方块在区块边缘，把相邻区块也标记为脏
            if local_y == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, -1, 0)); }
            if local_y == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(0, 1, 0)); }
            if local_x == 0 { dirty_chunks.push(chunk_pos + IVec3::new(-1, 0, 0)); }
            if local_x == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(1, 0, 0)); }
            if local_z == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, -1)); }
            if local_z == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, 1)); }

            // 重新渲染当前区块
            for (entity, chunk_comp, mut state) in &mut chunk_query {
                if dirty_chunks.contains(&chunk_comp.position) {
                    // commands.entity(entity)
                    //     .remove::<Mesh3d>()
                    //     .remove::<MeshMaterial3d<StandardMaterial>>();
                    //
                    // commands.entity(entity).despawn_children();

                    *state = ChunkState::DataReady;
                }
            }

            info!(
                "方块更新：坐标 {:?}, 物理ID变更为: {}, 触发了 {} 个相关区块网格同步重构。",
                target_pos, next_voxel_id, dirty_chunks.len()
            );
        }
    }
}