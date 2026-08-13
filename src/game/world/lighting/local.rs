//! 为可见区块和方块编辑区域提供高优先级局部光照重建。

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use bevy::prelude::*;

use super::chunk_light::ChunkLight;
use super::local_queue::LocalLightingQueue;
use super::rebuild::{
    BlockLightSource, GameLightInfo, LightingWorldSnapshot, rebuild_loaded_lighting,
};
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
const LOCAL_TARGET_COLUMN_BATCH_SIZE: usize = 24;
/// 玩家交互一次最多合并的水平列数，覆盖常用一至两圈传播半径。
const LOCAL_INTERACTION_COLUMN_BATCH_LIMIT: usize = 25;
/// 可见区块发现阶段保留的最大候选数，避免每个固定步扫描完整窗口。
const LOCAL_DISCOVERY_QUEUE_LIMIT: usize = 512;
/// 普通光照任务允许进入的任务池积压倍数；通道有限并发保证不会无界增长。
const LOCAL_TASK_BACKLOG_FACTOR: usize = 2;
/// 局部光照任务允许的最大并发数；列集由 pop_columns 从队列移除，天然不相交。
const LOCAL_LIGHTING_MAX_IN_FLIGHT: usize = 2;

struct LocalLightingBuildResult {
    session_id: u64,
    content_revision: u64,
    targets: Vec<IVec3>,
    dependency_halo: i32,
    snapshot: LightingWorldSnapshot,
    lights: HashMap<IVec3, ChunkLight>,
    sources: Vec<BlockLightSource>,
    priority: bool,
    sky_dirty: bool,
    waited_ticks: u16,
    elapsed: Duration,
}

/// 局部光照任务通道；限制为少量并发任务，连续编辑通过目标队列合并。
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

/// 把结构已稳定但光照快照尚未同步的可见区块推进到等待态并加入去重队列。
///
/// 该系统位于固定步光照结果提交之后、任务派发之前；旧网格保持可见，只有新网格
/// 会等待 `LightingReady`，从而避免新区块和编辑区块先黑闪再重建。
#[allow(clippy::too_many_arguments)]
pub(super) fn queue_pending_chunk_lighting(
    world: Res<WorldState>,
    lighting: Res<WorldLighting>,
    player_cache: Res<PlayerChunkCache>,
    streaming: Res<WorldStreamingConfig>,
    runtime: Res<ChunkRuntime>,
    mut states: Query<&mut ChunkState>,
    mut queue: ResMut<LocalLightingQueue>,
) {
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
        let Some(entity) = runtime.chunk_entity(position) else {
            continue;
        };
        let Ok(mut state) = states.get_mut(entity) else {
            continue;
        };
        if !state.has_completed_structure() {
            continue;
        }
        if lighting.is_chunk_light_current(position, data) {
            if matches!(
                *state,
                ChunkState::StructureReady | ChunkState::LightingPending
            ) {
                *state = ChunkState::LightingReady;
            }
        } else {
            if !matches!(*state, ChunkState::GeneratingMesh) {
                *state = ChunkState::LightingPending;
            }
            let needs_initial_sky = lighting
                .chunk_lights
                .get(&position)
                .is_none_or(|light| !light.is_initialized());
            if needs_initial_sky {
                queue.enqueue(position);
            } else {
                queue.enqueue_with_sky(position, false);
            }
        }
    }
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
    cached: Res<CachedLightInfo>,
    tracker: Res<LightingRebuildTracker>,
    lighting: Res<WorldLighting>,
    player_cache: Res<PlayerChunkCache>,
    chunk_runtime: Res<ChunkRuntime>,
    chunk_states: Query<&ChunkState>,
    task: Res<TaskManager>,
) {
    let mut edit_sky_dirty_columns = HashSet::<(i32, i32)>::new();
    let dependency_halo = cached.info.block_light_chunk_halo();
    let mut received_edit = false;
    for change in changed_blocks.read() {
        let sky_dirty = edit_affects_sky(&cached.info, &lighting, &world, change);

        if sky_dirty {
            let (cx, _cy, cz) = (
                change.world_pos.x.div_euclid(CHUNK_SIZE as i32),
                0,
                change.world_pos.z.div_euclid(CHUNK_SIZE as i32),
            );
            for x in cx - 1..=cx + 1 {
                for z in cz - 1..=cz + 1 {
                    edit_sky_dirty_columns.insert((x, z));
                }
            }
        }

        enqueue_block_change_targets(
            &world,
            change.world_pos,
            dependency_halo,
            sky_dirty,
            &mut queue,
        );
        received_edit = true;
    }
    if received_edit {
        queue.restart_edit_merge_window();
    }

    queue.age();
    if queue.wait_for_edit_merge() {
        return;
    }
    if channel.in_flight.load(Ordering::Relaxed) >= LOCAL_LIGHTING_MAX_IN_FLIGHT {
        return;
    }
    let interaction = queue.has_priority_target();
    let starvation_dispatch = queue.has_starved_target();
    if !interaction
        && !starvation_dispatch
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
    let mut sky_dirty = false;
    let mut any_interaction = false;
    let mut waited_ticks = 0;
    // 队列中带天光重建标记的列（含 requeue 目标），必须并入本轮脏列集合，
    // 否则已初始化列会因保留旧天光而残留邻域变化前的光。
    let mut selected_sky_dirty_columns = HashSet::<(i32, i32)>::new();
    for selected in queue.pop_columns(column_limit) {
        let (chunk_x, chunk_z) = selected.column;
        let mut column_targets = world
            .chunks()
            .map(|(position, _)| position)
            .filter(|position| position.x == chunk_x && position.z == chunk_z)
            .collect::<Vec<_>>();
        column_targets.sort_by_key(|position| position.y);
        if selected.is_starved() {
            column_targets.retain(|position| {
                chunk_generation_ready(*position, &chunk_runtime, &chunk_states)
            });
            sky_dirty |= selected.sky_dirty;
            if selected.sky_dirty {
                selected_sky_dirty_columns.insert((chunk_x, chunk_z));
            }
            any_interaction |= selected.priority;
            waited_ticks = waited_ticks.max(selected.waited_ticks);
            targets.extend(column_targets);
        } else if column_targets.iter().all(|position| {
            neighborhood_generation_ready(
                &world,
                *position,
                &chunk_runtime,
                &chunk_states,
                &player_cache,
                dependency_halo,
            )
        }) {
            sky_dirty |= selected.sky_dirty;
            if selected.sky_dirty {
                selected_sky_dirty_columns.insert((chunk_x, chunk_z));
            }
            any_interaction |= selected.priority;
            waited_ticks = waited_ticks.max(selected.waited_ticks);
            targets.extend(column_targets);
        } else {
            for position in column_targets.into_iter().rev() {
                queue.requeue(
                    position,
                    selected.waited_ticks,
                    selected.priority,
                    selected.sky_dirty,
                );
            }
        }
    }
    if targets.is_empty() {
        return;
    }
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
    // 只有未初始化天空光或玩家编辑改变天空通路的列需要重灌天光；
    // 已就绪列在任务内保留原有天空光，避免批内一个新区块拖垮整批。
    let mut sky_dirty_columns = HashSet::new();
    for (position, _) in snapshot.chunks() {
        if lighting
            .chunk_lights
            .get(&position)
            .is_none_or(|light| !light.is_initialized())
        {
            sky_dirty_columns.insert((position.x, position.z));
        }
    }
    sky_dirty_columns.extend(edit_sky_dirty_columns);
    sky_dirty_columns.extend(selected_sky_dirty_columns);
    sky_dirty |= !sky_dirty_columns.is_empty();
    let sender = channel.sender.clone();
    let in_flight = Arc::clone(&channel.in_flight);
    let info = cached.info.clone();
    // 始终保留已就绪列的天空光，只有脏列从空开始重灌；
    // 未就绪列在任务内由 reset 兜底，previous 缺失不会丢失正确性。
    let previous_lights = snapshot
        .chunks()
        .filter_map(|(position, _)| {
            if sky_dirty_columns.contains(&(position.x, position.z)) {
                return None;
            }
            lighting
                .chunk_lights
                .get(&position)
                .filter(|light| light.is_initialized())
                .map(|light| (position, Arc::clone(light)))
        })
        .collect::<HashMap<_, Arc<ChunkLight>>>();
    let content_revision = cached.revision;
    let session_id = tracker.session_id;

    channel.in_flight.fetch_add(1, Ordering::Relaxed);
    task.spawn_cpu(move || {
        let started = Instant::now();
        // 深拷贝留在 CPU 任务内，固定步只克隆 Arc，避免交互派发本身阻塞渲染。
        let mut lights = previous_lights
            .into_iter()
            .map(|(position, light)| (position, (*light).clone()))
            .collect::<HashMap<_, _>>();
        let sources = rebuild_loaded_lighting(&snapshot, &info, &mut lights, &sky_dirty_columns);
        let sent = sender.send(LocalLightingBuildResult {
            session_id,
            content_revision,
            targets,
            dependency_halo,
            snapshot,
            lights,
            sources,
            priority: any_interaction,
            sky_dirty,
            waited_ticks,
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
                queue.requeue(
                    *position,
                    result.waited_ticks,
                    result.priority,
                    result.sky_dirty || result.content_revision != cached.revision,
                );
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
            queue.requeue(
                *position,
                result.waited_ticks,
                result.priority,
                result.sky_dirty,
            );
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
        let position = components.position;
        let is_target = valid_targets.contains(&position);
        let needs_remesh = remesh.contains(&position);
        if !is_target && !needs_remesh {
            continue;
        }
        if needs_remesh {
            runtime.bump_revision(position);
        }
        if !state.has_completed_structure() {
            continue;
        }
        if is_target {
            *state = ChunkState::LightingReady;
        } else if needs_remesh {
            queue.enqueue_with_sky(position, result.sky_dirty);
            *state = ChunkState::LightingPending;
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
    sky_dirty: bool,
    queue: &mut LocalLightingQueue,
) {
    let center = world_to_chunk(world_pos);
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
        queue.prioritize_edit(position, sky_dirty);
    }
}

/// 只有方块的天空光透射发生变化时才需要重新灌入整列天光。
///
/// 发光属性变化只影响方块光；同透射材质间替换也不会改变天空通路。
fn edit_affects_sky(
    info: &GameLightInfo,
    lighting: &WorldLighting,
    world: &WorldState,
    change: &BlockChangedEvent,
) -> bool {
    const NEIGHBORS: [IVec3; 7] = [
        IVec3::ZERO,
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ];
    let old_filter = info.prop(change.old_block_id).filter;
    let new_filter = info.prop(change.new_block_id).filter;
    if old_filter == new_filter {
        return false;
    }
    if NEIGHBORS.into_iter().any(|offset| {
        lighting
            .light_cell_at_world(change.world_pos + offset)
            .is_some_and(|cell| !cell.sky.is_dark())
    }) {
        return true;
    }

    // 打开原本封闭的天井时，改动格及其邻域仍可能全黑；向上检查同列的已加载
    // 区块，只有确实能接到现有天空光柱时才升级为天光重建。未加载空间不会被当作露天。
    let opens_transmission = new_filter
        .iter()
        .zip(old_filter)
        .any(|(new, old)| *new > old);
    if !opens_transmission {
        return false;
    }
    let max_chunk_y = world
        .chunks()
        .filter(|(position, _)| {
            position.x == change.world_pos.x.div_euclid(CHUNK_SIZE as i32)
                && position.z == change.world_pos.z.div_euclid(CHUNK_SIZE as i32)
        })
        .map(|(position, _)| position.y)
        .max();
    let Some(max_chunk_y) = max_chunk_y else {
        return false;
    };
    let max_y = (max_chunk_y + 1) * CHUNK_SIZE as i32 - 1;
    let mut position = change.world_pos + IVec3::Y;
    while position.y <= max_y {
        if lighting
            .light_cell_at_world(position)
            .is_some_and(|cell| !cell.sky.is_dark())
        {
            return true;
        }
        position.y += 1;
    }
    false
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
                        matches!(*state, ChunkState::LoadFailed) || state.has_completed_structure()
                    })
        })
}

fn chunk_generation_ready(
    position: IVec3,
    runtime: &ChunkRuntime,
    states: &Query<&ChunkState>,
) -> bool {
    runtime
        .chunk_entity(position)
        .and_then(|entity| states.get(entity).ok())
        .is_some_and(|state| state.has_completed_structure())
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
