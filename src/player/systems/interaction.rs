use crate::player::systems::raycast::{raycast_voxel, TargetVoxel};
use crate::world::chunk::{ChunkComponents, ChunkState};
use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::core::input_block::InputBlocked;
use crate::core::state::inventory_ui_state::InventoryUiState;
use crate::player::components::Player;
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
    player_query: Query<Entity, With<Player>>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    mut break_events: MessageWriter<BlockBreakEvent>,
    mut place_events: MessageWriter<BlockPlaceEvent>,
    mut interact_events: MessageWriter<BlockInteractEvent>,
    mut sound_events: MessageWriter<BlockSoundEvent>,
    mut commands: Commands,
) {
    let Some(reg) = registry else { return; };
    // 当打开物品栏时不进行破坏和放置操作
    if input_blocked.0 { return; }

    let player_entity = player_query.single().ok();

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
                breaker: player_entity,
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
            mark_dirty_chunks(hit_pos, &mut chunk_query, &mut world_storage);
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
                        interactor: player_entity,
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
                placer: player_entity,
            });

            // 发送音效
            let prop = reg.get(block_id);
            sound_events.write(BlockSoundEvent {
                position: place_pos.as_vec3(),
                sound_material: prop.map(|p| p.sound.sound_material).unwrap_or_default(),
                action: SoundAction::Place,
                volume: prop.map(|p| p.sound.place_volume).unwrap_or(1.0),
            });

            mark_dirty_chunks(place_pos, &mut chunk_query, &mut world_storage);
        };
    }
}

fn mark_dirty_chunks(
    target_pos: IVec3,
    chunk_query: &mut Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    world_storage: &mut WorldStorage,
) {
    let chunk_pos = IVec3::new(
        target_pos.x.div_euclid(CHUNK_SIZE as i32),
        target_pos.y.div_euclid(CHUNK_SIZE as i32),
        target_pos.z.div_euclid(CHUNK_SIZE as i32),
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    // 记录当前区块修改时间
    world_storage.chunk_modified_times.insert(chunk_pos, now);

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

    // 边缘相邻区块也记录修改时间
    for &dirty_pos in &dirty_chunks {
        world_storage.chunk_modified_times.insert(dirty_pos, now);
    }

    for (_, chunk_comp, mut state) in chunk_query.iter_mut() {
        if dirty_chunks.contains(&chunk_comp.position) {
            *state = ChunkState::DataReady;
        }
    }
}
