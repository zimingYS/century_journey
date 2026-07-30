//! 执行树苗的低频确定性判定、完整空间预检和统一世界写入。

use super::runtime::{VegetationRuntime, chunk_is_ready, world_to_chunk_position};
use crate::content::block::event::BlockChangedEvent;
use crate::content::block::registry::BlockRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::content::vegetation::registry::TreeSpeciesRegistry;
use crate::game::simulation::SimulationRng;
use crate::game::world::block_ops::{get_voxel_at_world, set_voxel_at_world};
use crate::game::world::chunk::ChunkState;
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::structure::{TreeBlueprint, TreeBlueprintParameters};
use crate::game::world::time::{GameMinuteElapsed, WorldSimulationClock};
use crate::shared::random::RandomSource;
use crate::shared::tag::identifier::TagId;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// 树苗生长判定使用的独立随机域。
const TREE_GROWTH_RANDOM_DOMAIN: u64 = 0x5452_4545_4752_4F57;
/// 树形选择使用独立随机域，避免生长概率调整改变已经选定的树形。
const TREE_SHAPE_RANDOM_DOMAIN: u64 = 0x5452_4545_5348_4150;

/// 聚合树苗生长系统只读内容、区块状态和权威世界写入资源。
#[derive(SystemParam)]
pub(super) struct TreeGrowthContext<'w, 's> {
    clock: Res<'w, WorldSimulationClock>,
    simulation_rng: Res<'w, SimulationRng>,
    block_registry: Res<'w, BlockRegistry>,
    tag_registry: Res<'w, RuntimeTagRegistry>,
    species_registry: Res<'w, TreeSpeciesRegistry>,
    chunk_runtime: Res<'w, ChunkRuntime>,
    chunk_states: Query<'w, 's, &'static ChunkState>,
    world_state: ResMut<'w, WorldState>,
}

/// 在游戏分钟边界处理本分钟到期的树苗候选。
pub(super) fn grow_saplings_system(
    mut minute_events: MessageReader<GameMinuteElapsed>,
    context: TreeGrowthContext,
    mut runtime: ResMut<VegetationRuntime>,
    mut changed_blocks: MessageWriter<BlockChangedEvent>,
) {
    let TreeGrowthContext {
        clock,
        simulation_rng,
        block_registry,
        tag_registry,
        species_registry,
        chunk_runtime,
        chunk_states,
        mut world_state,
    } = context;
    if minute_events.read().count() == 0 {
        return;
    }

    let game_minute = clock.total_game_minutes();
    let candidates = runtime.sorted_candidates();
    for (position, sapling_block_id) in candidates {
        if get_voxel_at_world(position, &world_state) != sapling_block_id {
            runtime.remove_candidate(position);
            continue;
        }
        let Some(species) = species_registry.get_by_sapling_id(sapling_block_id) else {
            runtime.remove_candidate(position);
            continue;
        };
        let interval = species.definition.growth.attempt_interval_game_minutes;
        if !growth_attempt_is_due(position, sapling_block_id, game_minute, interval) {
            continue;
        }

        let event_key = SimulationRng::block_event_key(position, sapling_block_id);
        let mut chance_rng =
            simulation_rng.for_event(TREE_GROWTH_RANDOM_DOMAIN, game_minute, event_key);
        if chance_rng.next_f32() >= species.definition.growth.chance_per_attempt {
            continue;
        }

        let Some(support_tag) = block_registry
            .get(sapling_block_id)
            .and_then(|block| block.placement.required_support_tag.as_ref())
        else {
            continue;
        };
        let parameters = TreeBlueprintParameters {
            trunk_height_min: species.definition.blueprint.trunk_height.min,
            trunk_height_max: species.definition.blueprint.trunk_height.max,
            crown_radius_min: species.definition.blueprint.crown_radius.min,
            crown_radius_max: species.definition.blueprint.crown_radius.max,
        };
        let mut shape_rng = simulation_rng.for_event(TREE_SHAPE_RANDOM_DOMAIN, 0, event_key);
        let blueprint = TreeBlueprint::generate(
            position,
            shape_rng.next_u64() as u32,
            species.trunk_block_id,
            species.leaves_block_id,
            parameters,
        );

        let changes = try_apply_tree_growth(
            position,
            sapling_block_id,
            support_tag,
            &blueprint,
            &tag_registry,
            &mut world_state,
            |chunk_position| chunk_is_ready(chunk_position, &chunk_runtime, &chunk_states),
        );
        let Some(changes) = changes else {
            continue;
        };
        for change in changes {
            changed_blocks.write(change);
        }
        runtime.remove_candidate(position);
    }
}

fn growth_attempt_is_due(position: IVec3, block_id: u16, game_minute: u64, interval: u64) -> bool {
    debug_assert!(interval > 0);
    let phase = SimulationRng::block_event_key(position, block_id) % interval;
    game_minute % interval == phase
}

fn try_apply_tree_growth(
    anchor: IVec3,
    sapling_block_id: u16,
    support_tag: &TagId,
    blueprint: &TreeBlueprint,
    tag_registry: &RuntimeTagRegistry,
    world_state: &mut WorldState,
    mut chunk_is_ready: impl FnMut(IVec3) -> bool,
) -> Option<Vec<BlockChangedEvent>> {
    let support_position = anchor - IVec3::Y;
    let support_chunk = world_to_chunk_position(support_position);
    if !world_state.contains_chunk(support_chunk) || !chunk_is_ready(support_chunk) {
        return None;
    }
    let support_block_id = get_voxel_at_world(support_position, world_state);
    if !tag_registry.contains(support_tag, support_block_id) {
        return None;
    }

    let mut contains_anchor = false;
    for voxel in blueprint.voxels() {
        let target_chunk = world_to_chunk_position(voxel.world_pos);
        if !world_state.contains_chunk(target_chunk) || !chunk_is_ready(target_chunk) {
            return None;
        }
        let current_block_id = get_voxel_at_world(voxel.world_pos, world_state);
        if voxel.world_pos == anchor {
            contains_anchor = true;
            if current_block_id != sapling_block_id {
                return None;
            }
        } else if current_block_id != 0 {
            return None;
        }
    }
    if !contains_anchor {
        return None;
    }

    let mut changes = Vec::with_capacity(blueprint.voxels().len());
    for voxel in blueprint.voxels() {
        let change = set_voxel_at_world(voxel.world_pos, voxel.block_id, world_state)
            .expect("树形预检后，同一系统内的体素写入必须发生变化");
        changes.push(change);
    }
    Some(changes)
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/vegetation/growth.rs"]
mod tests;
