//! 在固定步的游戏分钟边界注册树苗，并推进已到期树木的生命周期阶段。

use super::runtime::{VegetationRuntime, chunk_is_ready, world_to_chunk_position};
use super::transition::{CurrentTreeForm, support_is_valid, try_apply_stage_transition};
use crate::content::block::registry::BlockRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::content::vegetation::definition::TreeBlueprintDefinition;
use crate::content::vegetation::registry::{RuntimeTreeSpecies, TreeSpeciesRegistry};
use crate::game::simulation::SimulationRng;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::chunk::ChunkState;
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::structure::{TreeBlueprint, TreeBlueprintParameters};
use crate::game::world::time::{GameMinuteElapsed, WorldSimulationClock};
use crate::game::world::vegetation::{TreeGrowthStage, TreeInstance};
use crate::shared::random::RandomSource;
use crate::shared::voxel_change::VoxelChangeBuffer;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// 树形选择使用独立随机域，使阶段时长调整不会改变既有树形。
const TREE_SHAPE_RANDOM_DOMAIN: u64 = 0x5452_4545_5348_4150;
/// 内容暂时缺失时延迟重查，避免每个游戏分钟反复访问同一无效实例。
const MISSING_CONTENT_RETRY_GAME_MINUTES: u64 = 60;

/// 聚合树木生命周期系统只读内容、区块状态和权威世界写入资源。
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

/// 在游戏分钟边界注册新树苗，并按根坐标稳定顺序推进到期实例。
pub(super) fn advance_tree_lifecycle_system(
    mut minute_events: MessageReader<GameMinuteElapsed>,
    context: TreeGrowthContext,
    mut runtime: ResMut<VegetationRuntime>,
    mut changes: ResMut<VoxelChangeBuffer>,
) {
    if minute_events.read().count() == 0 {
        return;
    }

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
    let game_minute = clock.total_game_minutes();

    // 候选索引只保存世界方块；首次分钟结算时才创建可持久化逻辑实例。
    for (position, sapling_block_id) in runtime.sorted_candidates() {
        if get_voxel_at_world(position, &world_state) != sapling_block_id {
            runtime.remove_candidate(position);
            continue;
        }
        if world_state.tree_instance(position).is_some() {
            continue;
        }
        let Some(species) = species_registry.get_by_sapling_id(sapling_block_id) else {
            runtime.remove_candidate(position);
            continue;
        };
        let Some(support_tag) = block_registry
            .get(sapling_block_id)
            .and_then(|block| block.placement.required_support_tag.as_ref())
        else {
            continue;
        };
        if !support_is_valid(
            position,
            support_tag,
            &tag_registry,
            &world_state,
            |chunk_position| chunk_is_ready(chunk_position, &chunk_runtime, &chunk_states),
        ) {
            continue;
        }

        let event_key = SimulationRng::block_event_key(position, sapling_block_id);
        let mut shape_rng = simulation_rng.for_event(TREE_SHAPE_RANDOM_DOMAIN, 0, event_key);
        let instance = TreeInstance::new_sapling(
            position,
            species.definition.identifier.clone(),
            shape_rng.next_u64() as u32,
            game_minute,
            species.definition.growth.sapling_duration_game_minutes,
        );
        world_state
            .insert_tree_instance(instance)
            .expect("树苗候选与实例主键已经在同一系统内完成校验");
        mark_tree_instance_modified(&mut world_state, position);
    }

    for root in world_state.due_tree_roots(game_minute) {
        let Some(instance) = world_state.tree_instance(root).cloned() else {
            continue;
        };
        let Some(species) = species_registry.get(instance.species()) else {
            defer_instance(
                &mut world_state,
                root,
                game_minute,
                MISSING_CONTENT_RETRY_GAME_MINUTES,
            );
            continue;
        };
        let Some(support_tag) = block_registry
            .get(species.sapling_block_id)
            .and_then(|block| block.placement.required_support_tag.as_ref())
        else {
            defer_instance(
                &mut world_state,
                root,
                game_minute,
                species.definition.growth.retry_interval_game_minutes,
            );
            continue;
        };

        let mature_parameters = blueprint_parameters(species.definition.blueprint);
        let young_parameters = species
            .definition
            .young_blueprint
            .map(blueprint_parameters)
            .unwrap_or_else(|| TreeBlueprintParameters::young_from_mature(mature_parameters));
        let young_blueprint = tree_blueprint(species, &instance, young_parameters);
        let (current_form, target_blueprint) = match instance.stage() {
            TreeGrowthStage::Sapling => (
                CurrentTreeForm::Sapling(species.sapling_block_id),
                young_blueprint.clone(),
            ),
            TreeGrowthStage::Young => (
                CurrentTreeForm::Blueprint(&young_blueprint),
                tree_blueprint(species, &instance, mature_parameters),
            ),
            TreeGrowthStage::Mature => continue,
        };

        if !try_apply_stage_transition(
            root,
            support_tag,
            current_form,
            &target_blueprint,
            &tag_registry,
            &world_state,
            &mut changes,
            |chunk_position| chunk_is_ready(chunk_position, &chunk_runtime, &chunk_states),
        ) {
            defer_instance(
                &mut world_state,
                root,
                game_minute,
                species.definition.growth.retry_interval_game_minutes,
            );
            continue;
        };

        let stored = world_state
            .tree_instance_mut(root)
            .expect("阶段体素提交期间不会删除同一树根实例");
        match instance.stage() {
            TreeGrowthStage::Sapling => {
                stored
                    .transition_to_young(
                        game_minute,
                        species.definition.growth.young_duration_game_minutes,
                    )
                    .expect("到期实例的快照阶段已经完成匹配");
                runtime.remove_candidate(root);
            }
            TreeGrowthStage::Young => stored
                .transition_to_mature(game_minute)
                .expect("到期实例的快照阶段已经完成匹配"),
            TreeGrowthStage::Mature => unreachable!("成熟树没有生命周期到期任务"),
        }
        mark_tree_instance_modified(&mut world_state, root);
    }
}

fn blueprint_parameters(definition: TreeBlueprintDefinition) -> TreeBlueprintParameters {
    TreeBlueprintParameters {
        trunk_height_min: definition.trunk_height.min,
        trunk_height_max: definition.trunk_height.max,
        crown_radius_min: definition.crown_radius.min,
        crown_radius_max: definition.crown_radius.max,
    }
}

fn tree_blueprint(
    species: &RuntimeTreeSpecies,
    instance: &TreeInstance,
    parameters: TreeBlueprintParameters,
) -> TreeBlueprint {
    TreeBlueprint::generate(
        instance.root(),
        instance.shape_seed(),
        species.trunk_block_id,
        species.leaves_block_id,
        parameters,
    )
}

fn defer_instance(
    world_state: &mut WorldState,
    root: IVec3,
    game_minute: u64,
    retry_interval_game_minutes: u64,
) {
    if let Some(instance) = world_state.tree_instance_mut(root) {
        instance
            .defer_update(game_minute, retry_interval_game_minutes)
            .expect("只有未成熟实例会进入生命周期重试队列");
        mark_tree_instance_modified(world_state, root);
    }
}

/// 把只修改树实例元数据的结算纳入增量自动存档。
///
/// 生命周期仍使用游戏分钟；这里的 Unix 时间只服务于存档新旧快照排序，不参与玩法判定。
fn mark_tree_instance_modified(world_state: &mut WorldState, root: IVec3) {
    let modified_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    world_state.mark_chunk_modified(world_to_chunk_position(root), modified_at);
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/vegetation/growth.rs"]
mod tests;
