use crate::player::systems::raycast::{raycast_voxel, TargetVoxel};
use crate::world::chunk::{ChunkComponents, ChunkState};
use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::core::input_block::InputBlocked;
use crate::core::state::inventory_ui_state::InventoryUiState;
use crate::voxel::behavior::{get_voxel_at_world, set_voxel_at_world};
use crate::voxel::event::{BlockBreakEvent, BlockInteractEvent, BlockPlaceEvent};
use crate::voxel::registry::BlockRegistry;
use crate::voxel::sound::{BlockSoundEvent, SoundAction};
use crate::world::storage::WorldStorage;

pub fn voxel_interaction_system(
    target_voxel: Res<TargetVoxel>,
    registry: Option<Res<BlockRegistry>>,
    input_blocked: Res<InputBlocked>,
    inventory_ui_state: Res<InventoryUiState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    mut break_events: MessageWriter<BlockBreakEvent>,
    mut place_events: MessageWriter<BlockPlaceEvent>,
    mut interact_events: MessageWriter<BlockInteractEvent>,
    mut sound_events: MessageWriter<BlockSoundEvent>,
    mut commands: Commands,
){
    let Some(reg) = registry else { return; };
    // 当打开物品栏时不进行破坏和放置操作
    if input_blocked.0 { return; }

    let left_click = mouse_button.just_pressed(MouseButton::Left);
    let right_click = mouse_button.just_pressed(MouseButton::Right);
    if !left_click && !right_click { return; }

    // 左键破坏，右键放置
    if let Some(ray_result) = &target_voxel.result {
        if left_click {
            // 左键破坏
            let hit_pos = ray_result.hit_pos;
            let hit_id = get_voxel_at_world(hit_pos, &world_storage);

            // 调用方块行为
            let behavior = reg.get_behavior_by_id(hit_id);
            behavior.on_break(hit_pos, hit_id, &mut world_storage, &mut commands);

            // 实际移除方块
            set_voxel_at_world(hit_pos, 0, &mut world_storage);

            // 发送破坏事件
            break_events.write(BlockBreakEvent {
                world_pos: hit_pos,
                block_id: hit_id,
                breaker: None, // TODO: 传入玩家实体
            });

            // 发送音效事件
            let prop = reg.get(hit_id);
            sound_events.write(BlockSoundEvent {
                position: hit_pos.as_vec3(),
                sound_material: prop.map(|p| p.sound.sound_material).unwrap_or_default(),
                action: SoundAction::Break,
                volume: prop.map(|p| p.sound.break_volume).unwrap_or(1.0),
            });

            // 标记脏区块
            mark_dirty_chunks(hit_pos, &mut chunk_query);
        } else {
            let hit_pos = ray_result.hit_pos;
            let hit_id = get_voxel_at_world(hit_pos, &world_storage);

            // 检查目标方块是否可交互
            if let Some(prop) = reg.get(hit_id) {
                if prop.is_interactable {
                    // 发送交互事件
                    interact_events.write(BlockInteractEvent {
                        world_pos: hit_pos,
                        block_id: hit_id,
                        face_normal: ray_result.normal,
                        interactor: None,
                    });

                    // 调用方块行为
                    let behavior = reg.get_behavior_by_id(hit_id);
                    behavior.on_interact(
                        hit_pos, hit_id, ray_result.normal, None,
                        &mut world_storage, &mut commands,
                    );

                    // 发送音效
                    sound_events.write(BlockSoundEvent {
                        position: hit_pos.as_vec3(),
                        sound_material: prop.sound.sound_material,
                        action: SoundAction::Step, // 交互音效用 step 类型
                        volume: 0.5,
                    });

                    // 交互型方块不放置新方块，直接返回
                    return;
                }
            }

            // 右键放置：从快捷栏里拿出当前手持方块的名称串唯一标识
            let place_pos = hit_pos + ray_result.normal;
            let current_hand_identifier = &inventory_ui_state.hotbar_items[inventory_ui_state.active_hotbar_index];
            // 翻译成运行时对应的动态ID
            let Some(block_id) = reg.get_id_by_identifier(current_hand_identifier) else { return; };
            if block_id == 0 { return; }

            // 调用方块行为的 on_place
            let behavior = reg.get_behavior_by_id(block_id);
            let allowed = behavior.on_place(
                place_pos, block_id, ray_result.normal,
                &mut world_storage, &mut commands,
            );
            if !allowed { return; }

            // 实际放置方块
            set_voxel_at_world(place_pos, block_id, &mut world_storage);

            // 发送放置事件
            place_events.write(BlockPlaceEvent {
                world_pos: place_pos,
                block_id,
                face_normal: ray_result.normal,
                placer: None,
            });

            // 发送音效
            let prop = reg.get(block_id);
            sound_events.write(BlockSoundEvent {
                position: place_pos.as_vec3(),
                sound_material: prop.map(|p| p.sound.sound_material).unwrap_or_default(),
                action: SoundAction::Place,
                volume: prop.map(|p| p.sound.place_volume).unwrap_or(1.0),
            });

            mark_dirty_chunks(place_pos, &mut chunk_query);
        };

        // // 获取击中的方块坐标
        // let chunk_pos = IVec3::new(
        //     target_pos.x.div_euclid(CHUNK_SIZE as i32),
        //     target_pos.y.div_euclid(CHUNK_SIZE as i32),
        //     target_pos.z.div_euclid(CHUNK_SIZE as i32),
        // );
        //
        // // 换算局部坐标
        // let local_x = target_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
        // let local_y = target_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
        // let local_z = target_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;
        //
        // if let Some(chunk_data) = world_storage.loaded_chunks.get_mut(&chunk_pos) {
        //     chunk_data.set_voxel(local_x, local_y, local_z, next_voxel_id);
        //
        //     let mut dirty_chunks = vec![chunk_pos];
        //     let max_idx = CHUNK_SIZE - 1;
        //
        //     // 若修改的方块在区块边缘，把相邻区块也标记为脏
        //     if local_y == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, -1, 0)); }
        //     if local_y == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(0, 1, 0)); }
        //     if local_x == 0 { dirty_chunks.push(chunk_pos + IVec3::new(-1, 0, 0)); }
        //     if local_x == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(1, 0, 0)); }
        //     if local_z == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, -1)); }
        //     if local_z == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, 1)); }
        //
        //     // 重新渲染当前区块
        //     for (entity, chunk_comp, mut state) in &mut chunk_query {
        //         if dirty_chunks.contains(&chunk_comp.position) {
        //             // commands.entity(entity)
        //             //     .remove::<Mesh3d>()
        //             //     .remove::<MeshMaterial3d<StandardMaterial>>();
        //             //
        //             // commands.entity(entity).despawn_children();
        //
        //             *state = ChunkState::DataReady;
        //         }
        //     }
        //
        //     info!(
        //         "方块更新：坐标 {:?}, 物理ID变更为: {}, 触发了 {} 个相关区块网格同步重构。",
        //         target_pos, next_voxel_id, dirty_chunks.len()
        //     );
        }
    }


fn mark_dirty_chunks(
    target_pos: IVec3,
    chunk_query: &mut Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let chunk_pos = IVec3::new(
        target_pos.x.div_euclid(CHUNK_SIZE as i32),
        target_pos.y.div_euclid(CHUNK_SIZE as i32),
        target_pos.z.div_euclid(CHUNK_SIZE as i32),
    );

    let local_x = target_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = target_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = target_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    let mut dirty_chunks = vec![chunk_pos];
    let max_idx = CHUNK_SIZE - 1;

    if local_y == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, -1, 0)); }
    if local_y == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(0, 1, 0)); }
    if local_x == 0 { dirty_chunks.push(chunk_pos + IVec3::new(-1, 0, 0)); }
    if local_x == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(1, 0, 0)); }
    if local_z == 0 { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, -1)); }
    if local_z == max_idx { dirty_chunks.push(chunk_pos + IVec3::new(0, 0, 1)); }

    for (_, chunk_comp, mut state) in chunk_query.iter_mut() {
        if dirty_chunks.contains(&chunk_comp.position) {
            *state = ChunkState::DataReady;
        }
    }
}
