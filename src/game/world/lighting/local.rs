//! 为可见区块和方块编辑区域提供高优先级局部光照重建。

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use bevy::prelude::*;

use super::chunk_light::ChunkLight;
use super::local_queue::LocalLightingQueue;
use super::rebuild::{BlockLightSource, LightingWorldSnapshot, rebuild_loaded_lighting};
use super::{CachedLightInfo, LightingRebuildTracker, WorldLighting, light_dependent_mesh_chunks};
use crate::content::block::definition::BlockLightDef;
use crate::content::block::event::BlockChangedEvent;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::world::chunk::{ChunkComponents, ChunkState};
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::streaming::{PlayerChunkCache, WorldStreamingConfig};
use crate::shared::voxel::CHUNK_SIZE;

/// 普通流送任务一次最多处理的核心水平列数。
const LOCAL_TARGET_COLUMN_BATCH_SIZE: usize = 8;
/// 玩家交互一次最多合并的水平列数，覆盖常用一至两圈传播半径。
const LOCAL_INTERACTION_COLUMN_BATCH_LIMIT: usize = 25;
/// 可见区块发现阶段保留的最大候选数，避免每个固定步扫描完整窗口。
const LOCAL_DISCOVERY_QUEUE_LIMIT: usize = 128;
/// 普通光照任务允许进入的任务池积压倍数；通道单飞保证不会无界增长。
const LOCAL_TASK_BACKLOG_FACTOR: usize = 2;

struct LocalLightingBuildResult {
    session_id: u64,
    content_revision: u64,
    targets: Vec<IVec3>,
    dependency_halo: i32,
    snapshot: LightingWorldSnapshot,
    lights: HashMap<IVec3, ChunkLight>,
    sources: Vec<BlockLightSource>,
    elapsed: Duration,
}

/// 局部光照任务通道；限制为单飞任务，连续编辑通过目标队列合并。
#[derive(Resource)]
pub(super) struct LocalLightingBuildChannel {
    sender: mpsc::Sender<LocalLightingBuildResult>,
    receiver: Mutex<mpsc::Receiver<LocalLightingBuildResult>>,
    in_flight: Arc<AtomicUsize>,
}

impl Default for LocalLightingBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}

/// 注册局部光照任务所需的会话期资源。
pub(super) fn register_resources(app: &mut App) {
    app.init_resource::<LocalLightingQueue>()
        .init_resource::<LocalLightingBuildChannel>();
}

/// 收集方块编辑和玩家附近尚无当前光照的区块，并派发一个局部传播任务。
///
/// Bevy 系统参数保持任务队列、流送优先级与区块生命周期的访问边界显式。
#[allow(clippy::too_many_arguments)]
pub(super) fn schedule_local_lighting_rebuild(
    mut changed_blocks: MessageReader<BlockChangedEvent>,
    mut queue: ResMut<LocalLightingQueue>,
    channel: Res<LocalLightingBuildChannel>,
    world: Res<WorldState>,
    world_generator: Res<WorldGenerator>,
    lighting: Res<WorldLighting>,
    cached: Res<CachedLightInfo>,
    tracker: Res<LightingRebuildTracker>,
    player_cache: Res<PlayerChunkCache>,
    streaming: Res<WorldStreamingConfig>,
    chunk_runtime: Res<ChunkRuntime>,
    chunk_states: Query<&ChunkState>,
    task: Res<TaskManager>,
) {
    let dependency_halo = cached.info.block_light_chunk_halo();
    for change in changed_blocks.read() {
        enqueue_block_change_targets(&world, change.world_pos, dependency_halo, &mut queue);
    }

    let Some(player_chunk) = player_cache.player_chunk_pos() else {
        return;
    };
    for &position in player_cache.ordered_chunks() {
        if queue.len() >= LOCAL_DISCOVERY_QUEUE_LIMIT {
            break;
        }
        if !streaming.should_mesh_chunk(player_chunk, position) {
            continue;
        }
        let Some(data) = world.chunk(position) else {
            continue;
        };
        let Some(entity) = chunk_runtime.chunk_entity(position) else {
            continue;
        };
        let Ok(state) = chunk_states.get(entity) else {
            continue;
        };
        if !matches!(
            *state,
            ChunkState::StructureReady | ChunkState::GeneratingMesh | ChunkState::Rendered
        ) || lighting.is_chunk_light_current(position, data)
        {
            continue;
        }
        queue.enqueue(position);
    }

    if channel.in_flight.load(Ordering::Relaxed) != 0 {
        return;
    }
    let interaction = queue.interaction_pending;
    if !interaction
        && task.pending_count()
            >= task
                .worker_count()
                .max(1)
                .saturating_mul(LOCAL_TASK_BACKLOG_FACTOR)
    {
        return;
    }
    let column_limit = local_column_batch_size(interaction, dependency_halo);
    let mut targets = Vec::new();
    for (chunk_x, chunk_z) in queue.pop_columns(column_limit) {
        let mut column_targets = world
            .chunks()
            .map(|(position, _)| position)
            .filter(|position| position.x == chunk_x && position.z == chunk_z)
            .collect::<Vec<_>>();
        column_targets.sort_by_key(|position| position.y);
        if column_targets.iter().all(|position| {
            neighborhood_generation_ready(
                &world,
                *position,
                &chunk_runtime,
                &chunk_states,
                &player_cache,
                dependency_halo,
            )
        }) {
            targets.extend(column_targets);
        } else {
            for position in column_targets.into_iter().rev() {
                if interaction {
                    queue.prioritize(position);
                } else {
                    queue.enqueue(position);
                }
            }
        }
    }
    if targets.is_empty() {
        return;
    }
    queue.interaction_pending = false;

    let columns = dependency_columns(&targets, dependency_halo);
    let snapshot = if world_generator.pipeline.biome_registry.is_empty() {
        LightingWorldSnapshot::from_columns(&world, &columns)
    } else {
        LightingWorldSnapshot::from_columns_with_terrain(
            &world,
            &columns,
            &world_generator.pipeline,
        )
    };
    let sender = channel.sender.clone();
    let in_flight = Arc::clone(&channel.in_flight);
    let info = cached.info.clone();
    let content_revision = cached.revision;
    let session_id = tracker.session_id;

    channel.in_flight.fetch_add(1, Ordering::Relaxed);
    task.spawn_cpu(move || {
        let started = Instant::now();
        let mut lights = HashMap::new();
        let sources = rebuild_loaded_lighting(&snapshot, &info, &mut lights);
        let sent = sender.send(LocalLightingBuildResult {
            session_id,
            content_revision,
            targets,
            dependency_halo,
            snapshot,
            lights,
            sources,
            elapsed: started.elapsed(),
        });
        if sent.is_err() {
            in_flight.fetch_sub(1, Ordering::Relaxed);
        }
        TaskResult::Success
    });
}

/// 提交仍匹配权威邻域的局部结果，并只重烘焙实际受影响的区块。
///
/// Bevy 系统参数保持光场、队列和区块生命周期的访问边界显式。
#[allow(clippy::too_many_arguments)]
pub(super) fn receive_local_lighting_results(
    mut lighting: ResMut<WorldLighting>,
    mut queue: ResMut<LocalLightingQueue>,
    channel: Res<LocalLightingBuildChannel>,
    world: Res<WorldState>,
    cached: Res<CachedLightInfo>,
    tracker: Res<LightingRebuildTracker>,
    task: Res<TaskManager>,
    mut runtime: ResMut<ChunkRuntime>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let result = {
        let receiver = channel
            .receiver
            .lock()
            .expect("局部光照结果通道互斥锁已损坏");
        receiver.try_recv().ok()
    };
    let Some(result) = result else {
        return;
    };
    channel.in_flight.fetch_sub(1, Ordering::Relaxed);

    if result.session_id != tracker.session_id || result.content_revision != cached.revision {
        if result.session_id == tracker.session_id {
            for position in &result.targets {
                queue.enqueue(*position);
            }
        }
        task.spawn_io(move || {
            drop(result);
            TaskResult::Success
        });
        return;
    }

    let valid_targets = result
        .targets
        .iter()
        .copied()
        .filter(|position| {
            result
                .snapshot
                .neighborhood_is_current(&world, *position, result.dependency_halo)
        })
        .collect::<HashSet<_>>();
    let requested_targets = result.targets.len();
    for position in &result.targets {
        if !valid_targets.contains(position) {
            queue.enqueue(*position);
        }
    }

    let LocalLightingBuildResult {
        snapshot,
        mut lights,
        sources,
        elapsed,
        ..
    } = result;
    let mut snapshots = snapshot.into_chunks();
    let mut affected = HashSet::new();
    let mut retired = Vec::new();

    for position in &valid_targets {
        let Some(light) = lights.remove(position) else {
            continue;
        };
        let Some(snapshot) = snapshots.remove(position) else {
            continue;
        };
        let was_current = world
            .chunk(*position)
            .is_some_and(|current| lighting.is_chunk_light_current(*position, current));
        let light_changed = lighting
            .chunk_lights
            .get(position)
            .is_none_or(|previous| !same_light(previous, &light));
        if let Some(previous) = lighting.chunk_lights.insert(*position, Arc::new(light)) {
            retired.push(previous);
        }
        lighting.chunk_snapshots.insert(*position, snapshot);
        if light_changed || !was_current {
            affected.insert(*position);
        }
    }

    if !valid_targets.is_empty() {
        lighting
            .sources
            .retain(|source| !valid_targets.contains(&world_to_chunk(source.world_pos)));
        lighting.sources.extend(
            sources
                .into_iter()
                .filter(|source| valid_targets.contains(&world_to_chunk(source.world_pos))),
        );
        lighting
            .sources
            .sort_by_key(|source| (source.world_pos.x, source.world_pos.y, source.world_pos.z));
        lighting.sources.dedup_by_key(|source| source.world_pos);
        lighting.revision = lighting.revision.wrapping_add(1);
    }

    let remesh = light_dependent_mesh_chunks(&affected);
    for (components, mut state) in &mut chunk_query {
        if !remesh.contains(&components.position) {
            continue;
        }
        runtime.bump_revision(components.position);
        if matches!(*state, ChunkState::GeneratingMesh | ChunkState::Rendered) {
            *state = ChunkState::StructureReady;
        }
    }

    task.spawn_io(move || {
        drop(lights);
        drop(snapshots);
        drop(retired);
        TaskResult::Success
    });
    debug!(
        "提交局部光照结果：{}/{} 个目标，{} 个区块重烘焙，计算耗时 {:.1} ms",
        valid_targets.len(),
        requested_targets,
        affected.len(),
        elapsed.as_secs_f64() * 1_000.0
    );
}

/// 方块事件发生后立即维护点光源索引，不等待体素光传播任务完成。
pub(super) fn sync_changed_block_sources(
    mut changed_blocks: MessageReader<BlockChangedEvent>,
    cached: Res<CachedLightInfo>,
    mut lighting: ResMut<WorldLighting>,
) {
    let mut changed = false;
    for event in changed_blocks.read() {
        let light = cached
            .info
            .prop(event.new_block_id)
            .light
            .filter(|light| light.emission > 0);
        changed |= update_source_entry(&mut lighting.sources, event.world_pos, light);
    }
    if changed {
        lighting.revision = lighting.revision.wrapping_add(1);
    }
}

/// 立即移除已卸载区块的光数组、权威快照和光源，避免持续流送累积内存。
pub(super) fn prune_unloaded_lighting(
    world: Res<WorldState>,
    mut lighting: ResMut<WorldLighting>,
    mut queue: ResMut<LocalLightingQueue>,
) {
    queue.retain_loaded(&world);
    let previous_lights = lighting.chunk_lights.len();
    let previous_snapshots = lighting.chunk_snapshots.len();
    let previous_sources = lighting.sources.len();
    lighting
        .chunk_lights
        .retain(|position, _| world.contains_chunk(*position));
    lighting
        .chunk_snapshots
        .retain(|position, _| world.contains_chunk(*position));
    lighting
        .sources
        .retain(|source| world.contains_chunk(world_to_chunk(source.world_pos)));
    if previous_lights != lighting.chunk_lights.len()
        || previous_snapshots != lighting.chunk_snapshots.len()
        || previous_sources != lighting.sources.len()
    {
        lighting.revision = lighting.revision.wrapping_add(1);
    }
}

/// 离开世界时清空尚未派发的局部目标；飞行中结果由会话编号拒绝。
pub(super) fn clear_local_lighting(mut queue: ResMut<LocalLightingQueue>) {
    queue.clear();
}

fn enqueue_block_change_targets(
    world: &WorldState,
    world_pos: IVec3,
    halo: i32,
    queue: &mut LocalLightingQueue,
) {
    let center = world_to_chunk(world_pos);
    queue.interaction_pending = true;
    let mut positions = world
        .chunks()
        .map(|(position, _)| position)
        .filter(|position| {
            (position.x - center.x).abs() <= halo && (position.z - center.z).abs() <= halo
        })
        .collect::<Vec<_>>();
    positions.sort_by_key(|position| {
        let delta = *position - center;
        (delta.x.abs() + delta.y.abs() + delta.z.abs(), delta.y.abs())
    });
    for position in positions.into_iter().rev() {
        queue.prioritize(position);
    }
}

fn dependency_columns(targets: &[IVec3], halo: i32) -> HashSet<(i32, i32)> {
    let mut columns = HashSet::new();
    for target in targets {
        for x in -halo..=halo {
            for z in -halo..=halo {
                columns.insert((target.x + x, target.z + z));
            }
        }
    }
    columns
}

fn local_column_batch_size(interaction: bool, dependency_halo: i32) -> usize {
    if !interaction {
        return LOCAL_TARGET_COLUMN_BATCH_SIZE;
    }
    let diameter = dependency_halo.max(0) as usize * 2 + 1;
    diameter
        .saturating_mul(diameter)
        .clamp(1, LOCAL_INTERACTION_COLUMN_BATCH_LIMIT)
}

fn neighborhood_generation_ready(
    world: &WorldState,
    target: IVec3,
    runtime: &ChunkRuntime,
    states: &Query<&ChunkState>,
    player_cache: &PlayerChunkCache,
    halo: i32,
) -> bool {
    player_cache
        .ordered_chunks()
        .iter()
        .copied()
        .filter(|position| {
            (position.x - target.x).abs() <= halo && (position.z - target.z).abs() <= halo
        })
        .all(|position| {
            world.contains_chunk(position)
                && runtime
                    .chunk_entity(position)
                    .and_then(|entity| states.get(entity).ok())
                    .is_some_and(|state| {
                        matches!(
                            *state,
                            ChunkState::LoadFailed
                                | ChunkState::StructureReady
                                | ChunkState::GeneratingMesh
                                | ChunkState::Rendered
                        )
                    })
        })
}

fn update_source_entry(
    sources: &mut Vec<BlockLightSource>,
    world_pos: IVec3,
    light: Option<BlockLightDef>,
) -> bool {
    let previous = sources
        .iter()
        .find(|source| source.world_pos == world_pos)
        .copied();
    let next = light.map(|light| BlockLightSource { world_pos, light });
    if previous == next {
        return false;
    }
    sources.retain(|source| source.world_pos != world_pos);
    if let Some(source) = next {
        sources.push(source);
        sources.sort_by_key(|source| (source.world_pos.x, source.world_pos.y, source.world_pos.z));
    }
    true
}

fn same_light(left: &ChunkLight, right: &ChunkLight) -> bool {
    left.is_initialized() == right.is_initialized() && left.fingerprint() == right.fingerprint()
}

fn world_to_chunk(position: IVec3) -> IVec3 {
    IVec3::new(
        position.x.div_euclid(CHUNK_SIZE as i32),
        position.y.div_euclid(CHUNK_SIZE as i32),
        position.z.div_euclid(CHUNK_SIZE as i32),
    )
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/lighting/local.rs"]
mod tests;
