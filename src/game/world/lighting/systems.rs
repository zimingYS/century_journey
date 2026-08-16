//! 固定步光照调度、结果提交与全局重建系统的实现。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use bevy::prelude::*;

use crate::content::block::registry::BlockRegistry;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::world::chunk::{ChunkComponents, ChunkState};
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::lighting::chunk_light::ChunkLight;
use crate::game::world::lighting::rebuild::{
    GameLightInfo, LightingWorldSnapshot, rebuild_loaded_lighting,
    rebuild_loaded_lighting_from_source_index,
};
use crate::game::world::lighting::resources::{
    CachedLightInfo, LightingBuildChannel, LightingBuildResult, LightingRebuildTracker,
    WorldLighting,
};
use crate::game::world::state::{ChunkRuntime, WorldState};

/// 内容注册表变化时重建传播使用的方块光属性快照。
pub(super) fn rebuild_light_info_snapshot(
    registry: Res<BlockRegistry>,
    mut cached: ResMut<CachedLightInfo>,
) {
    if registry.is_changed() {
        cached.info = GameLightInfo::from_registry(&registry);
        cached.revision = cached.revision.wrapping_add(1);
    }
}

/// 固定步收集区块拓扑、内容定义和方块事件，并按稳定批次派发后台重建。
///
/// 系统位于 `VoxelChange` 之后；全窗口传播只占用一个任务线程，连续更新会合并为
/// 下一次请求，避免主线程停顿和无界任务堆积。重建只重灌与权威体素快照不同步的
/// 水平列，已就绪列复用现有天光，避免每次世界稳定后全窗口重算直射天空光。
pub(super) fn schedule_lighting_rebuild(
    mut tracker: ResMut<LightingRebuildTracker>,
    world_state: Res<WorldState>,
    world_generator: Res<WorldGenerator>,
    cached: Res<CachedLightInfo>,
    channel: Res<LightingBuildChannel>,
    task: Res<TaskManager>,
    lighting: Res<WorldLighting>,
) {
    tracker.observe(world_state.snapshot_revision(), cached.revision);
    if !tracker.ready_to_dispatch(channel.in_flight.load(Ordering::Relaxed)) {
        return;
    }
    if tracker.should_defer_for_task_backlog(task.pending_count()) {
        return;
    }

    let snapshot = if world_generator.pipeline.biome_registry.is_empty() {
        LightingWorldSnapshot::from_world(&world_state)
    } else {
        LightingWorldSnapshot::from_world_with_terrain(&world_state, &world_generator.pipeline)
    };

    // 只有尚未与权威体素快照同步的列需要重灌直射天光；已就绪列保留现有天光。
    // 已就绪区块的发光方块索引由权威世界增量维护，全局校正复用索引，仅对尚未
    // 建立索引的新加载区块全量扫描，避免重复遍历整窗已就绪体素。
    let dirty_columns = snapshot
        .chunks()
        .filter(|(position, data)| !lighting.is_chunk_light_current(*position, data))
        .map(|(position, _)| (position.x, position.z))
        .collect::<HashSet<_>>();
    let previously_lit = snapshot
        .chunks()
        .filter(|(position, _)| {
            lighting
                .chunk_lights
                .get(position)
                .is_some_and(|light| light.is_initialized())
        })
        .map(|(position, _)| position)
        .collect::<HashSet<_>>();
    let scan_positions = snapshot
        .chunks()
        .filter(|(position, _)| !previously_lit.contains(position))
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    let indexed_sources = (!previously_lit.is_empty()).then(|| lighting.sources.clone());
    let previous_lights = lighting.chunk_lights.clone();

    let info = cached.info.clone();
    let content_revision = cached.revision;
    let world_revision = world_state.snapshot_revision();
    let session_id = tracker.session_id;
    let sender = channel.sender.clone();
    let in_flight = Arc::clone(&channel.in_flight);

    tracker.mark_dispatched();
    channel.in_flight.fetch_add(1, Ordering::Relaxed);
    task.spawn_cpu(move || {
        let started = Instant::now();
        // 深拷贝留在 CPU 任务内，固定步只克隆 Arc，避免派发阻塞渲染。
        let mut lights = previous_lights
            .into_iter()
            .map(|(position, light)| (position, (*light).clone()))
            .collect::<HashMap<_, _>>();
        let sources = if let Some(indexed_sources) = indexed_sources {
            rebuild_loaded_lighting_from_source_index(
                &snapshot,
                &info,
                &mut lights,
                &dirty_columns,
                &indexed_sources,
                &scan_positions,
            )
        } else {
            rebuild_loaded_lighting(&snapshot, &info, &mut lights, &dirty_columns)
        };
        let result = sender.send(LightingBuildResult {
            session_id,
            content_revision,
            world_revision,
            snapshot,
            lights,
            sources,
            elapsed: started.elapsed(),
        });
        if result.is_err() {
            in_flight.fetch_sub(1, Ordering::Relaxed);
        }
        TaskResult::Success
    });
}

/// 在固定步主线程提交最新且仍匹配权威世界的后台光照结果。
///
/// 提交只比较每区块预计算摘要；旧光数组的释放也交回任务池，避免大批区块同时
/// 替换时把内存回收成本重新带回渲染线程。系统参数显式标注权威资源与客户端
/// 区块状态的访问边界。
#[allow(clippy::too_many_arguments)]
pub(super) fn receive_lighting_results(
    mut lighting: ResMut<WorldLighting>,
    mut tracker: ResMut<LightingRebuildTracker>,
    world_state: Res<WorldState>,
    cached: Res<CachedLightInfo>,
    channel: Res<LightingBuildChannel>,
    task: Res<TaskManager>,
    mut runtime: ResMut<ChunkRuntime>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let result = {
        let receiver = channel.receiver.lock().expect("光照结果通道互斥锁已损坏");
        receiver.try_recv().ok()
    };
    let Some(result) = result else {
        return;
    };
    channel.in_flight.fetch_sub(1, Ordering::Relaxed);

    if !lighting_result_is_current(&result, &tracker, &cached, &world_state) {
        tracker.pending = true;
        tracker.urgent |= result.content_revision != cached.revision;
        let elapsed = result.elapsed;
        task.spawn_io(move || {
            drop(result);
            TaskResult::Success
        });
        debug!(
            "丢弃过期光照结果：计算耗时 {:.1} ms",
            elapsed.as_secs_f64() * 1_000.0
        );
        return;
    }

    let LightingBuildResult {
        snapshot,
        lights: rebuilt_lights,
        sources,
        elapsed,
        ..
    } = result;
    let lights = rebuilt_lights
        .into_iter()
        .map(|(position, light)| (position, Arc::new(light)))
        .collect::<HashMap<_, _>>();
    let affected = changed_light_chunks(&lighting.chunk_lights, &lights);
    let remesh = light_dependent_mesh_chunks(&affected);
    let retired_lights = std::mem::replace(&mut lighting.chunk_lights, lights);
    lighting.sources = sources;
    lighting.revision = lighting.revision.wrapping_add(1);
    lighting.chunk_snapshots = snapshot.into_chunks();

    for (components, mut state) in &mut chunk_query {
        let position = components.position;
        let needs_remesh = remesh.contains(&position);
        let light_is_current = world_state
            .chunk(position)
            .is_some_and(|data| lighting.is_chunk_light_current(position, data));
        if !needs_remesh
            && !matches!(
                *state,
                ChunkState::StructureReady | ChunkState::LightingPending
            )
        {
            continue;
        }
        if needs_remesh {
            runtime.bump_revision(position);
        }
        if state.has_completed_structure() {
            *state = if light_is_current {
                ChunkState::LightingReady
            } else {
                ChunkState::LightingPending
            };
        }
    }

    if !retired_lights.is_empty() {
        task.spawn_io(move || {
            drop(retired_lights);
            TaskResult::Success
        });
    }
    debug!(
        "提交后台光照结果：{} 个区块变化，计算耗时 {:.1} ms",
        affected.len(),
        elapsed.as_secs_f64() * 1_000.0
    );
}

/// 判断后台结果是否仍匹配当前权威世界与内容修订。
pub(super) fn lighting_result_is_current(
    result: &LightingBuildResult,
    tracker: &LightingRebuildTracker,
    cached: &CachedLightInfo,
    world: &WorldState,
) -> bool {
    result.session_id == tracker.session_id
        && result.content_revision == cached.revision
        && result.world_revision == world.snapshot_revision()
}

/// 离开世界时清除可重建光场，确保相同区块坐标的新会话不会复用旧快照。
pub(super) fn clear_world_lighting(
    mut lighting: ResMut<WorldLighting>,
    mut tracker: ResMut<LightingRebuildTracker>,
    channel: Res<LightingBuildChannel>,
    task: Res<TaskManager>,
) {
    let retired_lights = std::mem::take(&mut lighting.chunk_lights);
    lighting.chunk_snapshots.clear();
    lighting.sources.clear();
    lighting.revision = lighting.revision.wrapping_add(1);
    let session_id = tracker.session_id.wrapping_add(1);
    *tracker = LightingRebuildTracker {
        session_id,
        ..Default::default()
    };

    let receiver = channel.receiver.lock().expect("光照结果通道互斥锁已损坏");
    let mut retired_results = Vec::new();
    while let Ok(result) = receiver.try_recv() {
        channel.in_flight.fetch_sub(1, Ordering::Relaxed);
        retired_results.push(result);
    }
    drop(receiver);

    if !retired_lights.is_empty() || !retired_results.is_empty() {
        task.spawn_io(move || {
            drop(retired_lights);
            drop(retired_results);
            TaskResult::Success
        });
    }
}

/// 比较新旧光数组，返回实际发生变化的区块集合。
pub(super) fn changed_light_chunks(
    previous: &HashMap<IVec3, Arc<ChunkLight>>,
    rebuilt: &HashMap<IVec3, Arc<ChunkLight>>,
) -> HashSet<IVec3> {
    previous
        .keys()
        .chain(rebuilt.keys())
        .copied()
        .filter(
            |position| match (previous.get(position), rebuilt.get(position)) {
                (Some(previous), Some(rebuilt)) => {
                    previous.is_initialized() != rebuilt.is_initialized()
                        || previous.fingerprint() != rebuilt.fingerprint()
                }
                (None, None) => false,
                _ => true,
            },
        )
        .collect()
}

/// 光数组变化还会影响六个邻居烘焙的边界面，因此这些网格必须一起失效。
pub(super) fn light_dependent_mesh_chunks(changed: &HashSet<IVec3>) -> HashSet<IVec3> {
    const NEIGHBORS: [IVec3; 6] = [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ];
    let mut affected = HashSet::with_capacity(changed.len().saturating_mul(7));
    for position in changed {
        affected.insert(*position);
        affected.extend(NEIGHBORS.map(|offset| *position + offset));
    }
    affected
}
