use crate::content::block::event::{BlockBreakEvent, BlockInteractEvent, BlockPlaceEvent};
use crate::content::block::registry::BlockRegistry;
use crate::content::block::sound::{BlockSoundEvent, SoundAction};
use crate::content::constant::world::CHUNK_SIZE;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::loot::block_registry::BlockLootRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::block::BlockBehaviorRegistry;
use crate::game::gameplay::block_action::{
    BlockBreakProgress, BlockBreakState, active_tool_data, block_break_seconds, can_break_block,
    can_place_block, consume_placed_block_item, is_replaceable_block,
};
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::item::stack::ToolDamageResult;
use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::{Player, PlayerCollider};
use crate::game::player::systems::raycast::TargetVoxel;
use crate::game::world::block_ops::{get_voxel_at_world, set_voxel_at_world};
use crate::game::world::chunk::{ChunkComponents, ChunkState};
use crate::game::world::entity::dropped_item::{
    DroppedItemVelocity, spawn_dropped_item_with_velocity,
};
use crate::game::world::storage::WorldStorage;
use crate::game::world::systems::break_pipeline::execute_block_break;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(SystemParam)]
pub struct VoxelEventWriters<'w> {
    pub break_events: MessageWriter<'w, BlockBreakEvent>,
    pub place_events: MessageWriter<'w, BlockPlaceEvent>,
    pub interact_events: MessageWriter<'w, BlockInteractEvent>,
    pub sound_events: MessageWriter<'w, BlockSoundEvent>,
}

#[derive(SystemParam)]
pub struct BlockBreakRuntime<'w> {
    pub time: Res<'w, Time>,
    pub state: ResMut<'w, BlockBreakState>,
    pub progress: ResMut<'w, BlockBreakProgress>,
}

pub fn voxel_interaction_system(
    target_voxel: Res<TargetVoxel>,
    registry: Option<Res<BlockRegistry>>,
    item_registry: Option<Res<ItemRegistry>>,
    behavior_registry: Res<BlockBehaviorRegistry>,
    actions: Res<PlayerActionState>,
    mut inventory_state: ResMut<InventoryState>,
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    player_query: Query<(Entity, &Transform, &PlayerCollider), With<Player>>,
    gamemode: Res<PlayerGameMode>,
    loot_registry: Option<Res<BlockLootRegistry>>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    mut events: VoxelEventWriters,
    mut break_runtime: BlockBreakRuntime,
    mut commands: Commands,
) {
    let Some(reg) = registry else {
        break_runtime.state.clear();
        break_runtime.progress.clear();
        return;
    };
    let break_active = break_action_active(&actions, &gamemode);
    let right_click =
        actions.just_pressed(PlayerAction::Use) || actions.just_pressed(PlayerAction::PlaceBlock);

    if !break_active {
        break_runtime.state.clear();
        break_runtime.progress.clear();
    }
    if !break_active && !right_click {
        return;
    }

    let Some(ray_result) = &target_voxel.result else {
        break_runtime.state.clear();
        break_runtime.progress.clear();
        return;
    };

    let player_entity = player_query.iter().next().map(|(entity, _, _)| entity);

    if break_active {
        let hit_pos = ray_result.hit_pos;
        let hit_id = get_voxel_at_world(hit_pos, &world_storage);

        if !can_break_block(hit_id, &gamemode, tag_registry.as_deref()) {
            break_runtime.state.clear();
            break_runtime.progress.clear();
            return;
        }

        let Some(prop) = reg.get(hit_id) else {
            break_runtime.state.clear();
            break_runtime.progress.clear();
            return;
        };

        let active_stack = inventory_state.hotbar.active_stack();
        let active_tool = active_tool_data(active_stack, item_registry.as_deref());
        let active_tool_max_durability = active_tool.map(|tool| tool.max_durability);
        let active_item = inventory_state.hotbar.active_item().clone();

        let Some(required_seconds) = block_break_seconds(prop, &gamemode, active_tool) else {
            break_runtime.state.clear();
            break_runtime.progress.clear();
            return;
        };

        if required_seconds > 0.0 {
            let elapsed = break_runtime.state.tick(
                hit_pos,
                hit_id,
                &active_item,
                break_runtime.time.delta_secs(),
            );
            let progress = elapsed / required_seconds;
            break_runtime.progress.set(hit_pos, hit_id, progress);

            if progress < 1.0 {
                return;
            }
        }

        let broke_block = execute_block_break(
            hit_pos,
            hit_id,
            &gamemode,
            tag_registry.as_deref(),
            active_tool,
            &reg,
            &behavior_registry,
            loot_registry.as_deref(),
            &mut world_storage,
            &mut commands,
        );

        break_runtime.state.clear();
        break_runtime.progress.clear();

        if !broke_block {
            return;
        }

        if gamemode.is_survival()
            && let Some(max_durability) = active_tool_max_durability
        {
            let slot = inventory_state.hotbar.active_stack_mut();
            if let Some(stack) = slot.as_mut()
                && stack.damage_tool(1, max_durability) == ToolDamageResult::Broken
            {
                *slot = None;
                log::info!("[工具] 当前工具耐久耗尽并损坏");
            }
        }

        events.break_events.write(BlockBreakEvent {
            world_pos: hit_pos,
            block_id: hit_id,
            breaker: player_entity,
        });

        events.sound_events.write(BlockSoundEvent {
            position: hit_pos.as_vec3(),
            sound_material: prop.sound.sound_material,
            action: SoundAction::Break,
            volume: prop.sound.break_volume,
        });

        mark_dirty_chunks(hit_pos, &mut chunk_query, &mut world_storage);
        return;
    }

    let hit_pos = ray_result.hit_pos;
    let hit_id = get_voxel_at_world(hit_pos, &world_storage);

    if let Some(prop) = reg.get(hit_id)
        && prop.is_interactable
    {
        events.interact_events.write(BlockInteractEvent {
            world_pos: hit_pos,
            block_id: hit_id,
            face_normal: ray_result.normal,
            interactor: player_entity,
        });

        let behavior = behavior_registry.get_behavior_by_id(hit_id, &reg);
        behavior.on_interact(
            hit_pos,
            hit_id,
            ray_result.normal,
            None,
            &mut world_storage,
            &mut commands,
        );

        events.sound_events.write(BlockSoundEvent {
            position: hit_pos.as_vec3(),
            sound_material: prop.sound.sound_material,
            action: SoundAction::Interact,
            volume: 0.5,
        });
        return;
    }

    let place_pos = hit_pos + ray_result.normal;
    let existing_id = get_voxel_at_world(place_pos, &world_storage);
    if !is_replaceable_block(existing_id, tag_registry.as_deref()) {
        return;
    }

    let current_hand_item = inventory_state.hotbar.active_item();
    let current_hand_identifier: String = item_registry
        .as_ref()
        .and_then(|ir| ir.block_identifier(current_hand_item))
        .map(|id| id.to_string())
        .unwrap_or_else(|| "century_journey:air".to_string());

    let Some(block_id) = reg.get_id_by_identifier(&current_hand_identifier) else {
        return;
    };
    if !can_place_block(
        block_id,
        &gamemode,
        Some(inventory_state.hotbar.active_stack()),
    ) {
        return;
    }

    // 不允许把方块放进玩家碰撞箱；否则下一帧的脱困逻辑会被迫移动玩家。
    if player_query.iter().any(|(_, transform, collider)| {
        voxel_intersects_player(place_pos, transform.translation, collider.half_extents)
    }) {
        return;
    }

    let behavior = behavior_registry.get_behavior_by_id(block_id, &reg);
    let allowed = behavior.on_place(
        place_pos,
        block_id,
        ray_result.normal,
        &mut world_storage,
        &mut commands,
    );
    if !allowed {
        return;
    }

    if !consume_placed_block_item(&mut inventory_state, &gamemode) {
        return;
    }

    set_voxel_at_world(place_pos, block_id, &mut world_storage);

    events.place_events.write(BlockPlaceEvent {
        world_pos: place_pos,
        block_id,
        face_normal: ray_result.normal,
        placer: player_entity,
    });

    let prop = reg.get(block_id);
    events.sound_events.write(BlockSoundEvent {
        position: place_pos.as_vec3(),
        sound_material: prop.map(|p| p.sound.sound_material).unwrap_or_default(),
        action: SoundAction::Place,
        volume: prop.map(|p| p.sound.place_volume).unwrap_or(1.0),
    });

    mark_dirty_chunks(place_pos, &mut chunk_query, &mut world_storage);
}

/// 创造模式的瞬间破坏只接受按下边沿，避免一次鼠标点击跨帧破坏整列方块。
fn break_action_active(actions: &PlayerActionState, gamemode: &PlayerGameMode) -> bool {
    if gamemode.is_creative() {
        actions.just_pressed(PlayerAction::BreakBlock)
    } else {
        actions.pressed(PlayerAction::BreakBlock)
    }
}

fn voxel_intersects_player(voxel: IVec3, player_position: Vec3, half_extents: Vec3) -> bool {
    let block_min = voxel.as_vec3();
    let block_max = block_min + Vec3::ONE;
    let player_min = player_position - half_extents;
    let player_max = player_position + half_extents;

    player_min.x < block_max.x
        && player_max.x > block_min.x
        && player_min.y < block_max.y
        && player_max.y > block_min.y
        && player_min.z < block_max.z
        && player_max.z > block_min.z
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

    world_storage.chunk_modified_times.insert(chunk_pos, now);

    let local_x = target_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = target_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = target_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    let mut dirty_chunks = vec![chunk_pos];
    let max_idx = CHUNK_SIZE - 1;

    if local_y == 0 {
        dirty_chunks.push(chunk_pos + IVec3::new(0, -1, 0));
    }
    if local_y == max_idx {
        dirty_chunks.push(chunk_pos + IVec3::new(0, 1, 0));
    }
    if local_x == 0 {
        dirty_chunks.push(chunk_pos + IVec3::new(-1, 0, 0));
    }
    if local_x == max_idx {
        dirty_chunks.push(chunk_pos + IVec3::new(1, 0, 0));
    }
    if local_z == 0 {
        dirty_chunks.push(chunk_pos + IVec3::new(0, 0, -1));
    }
    if local_z == max_idx {
        dirty_chunks.push(chunk_pos + IVec3::new(0, 0, 1));
    }

    for &dirty_pos in &dirty_chunks {
        world_storage.chunk_modified_times.insert(dirty_pos, now);
    }

    for (_, chunk_comp, mut state) in chunk_query.iter_mut() {
        if dirty_chunks.contains(&chunk_comp.position) {
            *state = ChunkState::StructureReady;
        }
    }
}

pub fn drop_item_system(
    mut reader: MessageReader<crate::game::inventory::events::DropItemEvent>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let forward = player_transform.forward().as_vec3();
    let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z);
    let throw_direction = if horizontal_forward.length_squared() > 0.0001 {
        horizontal_forward.normalize()
    } else {
        Vec3::Z
    };

    for event in reader.read() {
        if event.stack.is_empty() {
            continue;
        }

        let pos = player_transform.translation + Vec3::Y * 1.25 + throw_direction * 0.75;
        let velocity = DroppedItemVelocity::thrown(throw_direction);
        spawn_dropped_item_with_velocity(&mut commands, pos, event.stack.clone(), velocity);
        log::info!("[Q] Dropped {:?}", event.stack);
    }
}

pub fn drop_active_hotbar_action_system(
    actions: Res<PlayerActionState>,
    mut inventory: ResMut<InventoryState>,
    mut writer: MessageWriter<crate::game::inventory::events::DropItemEvent>,
) {
    if !actions.just_pressed(PlayerAction::DropItem) {
        return;
    }
    let amount = if actions.pressed(PlayerAction::Sprint) {
        u32::MAX
    } else {
        1
    };
    let slot = inventory.hotbar.active_stack_mut();
    let Some(stack) = slot.as_mut() else {
        return;
    };
    let dropped = stack.take(amount);
    if stack.is_empty() {
        *slot = None;
    }
    if let Some(stack) = dropped {
        writer.write(crate::game::inventory::events::DropItemEvent { stack });
    }
}

#[cfg(test)]
mod placement_tests {
    use super::*;
    use crate::game::gameplay::gamemode::GameMode;

    #[test]
    fn creative_break_fix_held_click_only_breaks_on_first_frame() {
        let creative = PlayerGameMode {
            mode: GameMode::Creative,
        };
        let survival = PlayerGameMode::default();
        let mut actions = PlayerActionState::default();

        actions.update(true, [PlayerAction::BreakBlock]);
        assert!(break_action_active(&actions, &creative));

        actions.update(true, [PlayerAction::BreakBlock]);
        assert!(!break_action_active(&actions, &creative));
        assert!(break_action_active(&actions, &survival));
    }

    #[test]
    fn requested_fix_block_inside_player_is_rejected() {
        let half = Vec3::new(0.3, 0.9, 0.3);
        let standing_position = Vec3::new(0.5, 10.9, 0.5);

        assert!(voxel_intersects_player(
            IVec3::new(0, 10, 0),
            standing_position,
            half
        ));
        assert!(!voxel_intersects_player(
            IVec3::new(0, 9, 0),
            standing_position,
            half
        ));
    }
}
