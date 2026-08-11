//! 组织权威光照数据与方块光传播规则。
//!
//! 光级数组（`ChunkLight`）是会话期世界状态：固定步负责版本与提交，任务线程
//! 执行局部优先传播和低频全局校正，客户端网格通过快照消费；不进存档。

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::content::block::registry::BlockRegistry;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::simulation::SimulationSet;
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::lighting::chunk_light::{ChunkLight, LightCell};
use crate::game::world::lighting::rebuild::{
    BlockLightSource, GameLightInfo, LightingWorldSnapshot, rebuild_loaded_lighting,
};
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::shared::voxel::CHUNK_SIZE;

pub mod chunk_light;
mod local;
mod local_queue;
pub mod rebuild;

/// 已加载区块的光级数组（会话期态，不进存档）。
#[derive(Resource, Default)]
pub struct WorldLighting {
    /// 区块坐标 -> 光级数组。
    pub chunk_lights: HashMap<IVec3, Arc<ChunkLight>>,
    /// 当前已加载窗口内的有序发光方块索引，供客户端选择实体光源。
    pub sources: Vec<BlockLightSource>,
    /// 每次局部、全局或光源索引提交后递增，供客户端缓存判断表现是否需要同步。
    pub revision: u64,
    /// 上次重建消费的权威区块快照；保留 Arc 可可靠识别同坐标数据替换与原地写入。
    chunk_snapshots: HashMap<IVec3, Arc<ChunkData>>,
}

impl WorldLighting {
    /// 判断指定光数组是否对应当前权威区块快照。
    pub fn is_chunk_light_current(&self, position: IVec3, data: &Arc<ChunkData>) -> bool {
        self.chunk_snapshots
            .get(&position)
            .is_some_and(|snapshot| Arc::ptr_eq(snapshot, data))
    }

    /// 读取整数世界坐标处已初始化的天空光和方块光。
    pub fn light_cell_at_world(&self, position: IVec3) -> Option<LightCell> {
        let chunk_position = IVec3::new(
            position.x.div_euclid(CHUNK_SIZE as i32),
            position.y.div_euclid(CHUNK_SIZE as i32),
            position.z.div_euclid(CHUNK_SIZE as i32),
        );
        let local = IVec3::new(
            position.x.rem_euclid(CHUNK_SIZE as i32),
            position.y.rem_euclid(CHUNK_SIZE as i32),
            position.z.rem_euclid(CHUNK_SIZE as i32),
        );
        self.chunk_lights
            .get(&chunk_position)
            .filter(|light| light.is_initialized())
            .map(|light| light.get(local.x as usize, local.y as usize, local.z as usize))
    }
}

/// 内容注册表变化时重建的传播属性快照。
#[derive(Resource, Default)]
pub struct CachedLightInfo {
    info: GameLightInfo,
    revision: u64,
}

/// 区块流送输入稳定后再做低频全局校正，交互和可见区块由局部任务先处理。
#[derive(Resource, Default)]
struct LightingRebuildTracker {
    observed_world: Vec<(IVec3, usize)>,
    observed_content_revision: u64,
    stable_ticks: u8,
    pending: bool,
    urgent: bool,
    session_id: u64,
}

impl LightingRebuildTracker {
    fn observe(&mut self, world_signature: Vec<(IVec3, usize)>, content_revision: u64) {
        if self.observed_world != world_signature {
            self.observed_world = world_signature;
            self.stable_ticks = 0;
            self.pending = true;
        } else if self.pending {
            self.stable_ticks = self.stable_ticks.saturating_add(1);
        }

        if self.observed_content_revision != content_revision {
            self.observed_content_revision = content_revision;
            self.pending = true;
            self.urgent = true;
        }
    }

    fn ready_to_dispatch(&self, in_flight: usize) -> bool {
        in_flight == 0
            && self.pending
            && (self.urgent || self.stable_ticks >= WORLD_REBUILD_STABLE_TICKS)
    }

    fn mark_dispatched(&mut self) {
        self.pending = false;
        self.urgent = false;
        self.stable_ticks = 0;
    }
}

/// 后台重建完成后一次性交还给固定步提交系统的数据。
struct LightingBuildResult {
    session_id: u64,
    content_revision: u64,
    snapshot: LightingWorldSnapshot,
    lights: HashMap<IVec3, ChunkLight>,
    sources: Vec<BlockLightSource>,
    elapsed: Duration,
}

/// 限制全窗口光照最多只有一个后台任务，并负责跨线程移交结果。
#[derive(Resource)]
struct LightingBuildChannel {
    sender: mpsc::Sender<LightingBuildResult>,
    receiver: Mutex<mpsc::Receiver<LightingBuildResult>>,
    in_flight: Arc<AtomicUsize>,
}

impl Default for LightingBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}

/// 连续一秒没有新快照后才做全局校正；交互区域由局部任务即时处理。
const WORLD_REBUILD_STABLE_TICKS: u8 = 20;

/// 组装权威光照数据与固定步传播系统。
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        local::register_resources(app);
        app.init_resource::<WorldLighting>()
            .init_resource::<CachedLightInfo>()
            .init_resource::<LightingRebuildTracker>()
            .init_resource::<LightingBuildChannel>()
            .add_systems(
                FixedUpdate,
                (
                    rebuild_light_info_snapshot,
                    local::prune_unloaded_lighting,
                    local::receive_local_lighting_results,
                    receive_lighting_results,
                    local::sync_changed_block_sources,
                    local::schedule_local_lighting_rebuild,
                    schedule_lighting_rebuild,
                )
                    .chain()
                    .after(SimulationSet::VoxelChange)
                    .before(SimulationSet::Survival),
            )
            .add_systems(
                OnExit(crate::shared::states::AppState::InGame),
                (clear_world_lighting, local::clear_local_lighting).chain(),
            );
    }
}

/// 内容注册表变化时重建传播使用的方块光属性快照。
fn rebuild_light_info_snapshot(registry: Res<BlockRegistry>, mut cached: ResMut<CachedLightInfo>) {
    if registry.is_changed() {
        cached.info = GameLightInfo::from_registry(&registry);
        cached.revision = cached.revision.wrapping_add(1);
    }
}

/// 固定步收集区块拓扑、内容定义和方块事件，并按稳定批次派发后台重建。
///
/// 系统位于 `VoxelChange` 之后；全窗口传播只占用一个任务线程，连续更新会合并为
/// 下一次请求，避免主线程停顿和无界任务堆积。
fn schedule_lighting_rebuild(
    mut tracker: ResMut<LightingRebuildTracker>,
    world_state: Res<WorldState>,
    world_generator: Res<WorldGenerator>,
    cached: Res<CachedLightInfo>,
    channel: Res<LightingBuildChannel>,
    task: Res<TaskManager>,
) {
    let world_signature = current_world_signature(&world_state);
    tracker.observe(world_signature, cached.revision);
    if !tracker.ready_to_dispatch(channel.in_flight.load(Ordering::Relaxed)) {
        return;
    }
    if task.pending_count() != 0 {
        return;
    }

    let snapshot = if world_generator.pipeline.biome_registry.is_empty() {
        LightingWorldSnapshot::from_world(&world_state)
    } else {
        LightingWorldSnapshot::from_world_with_terrain(&world_state, &world_generator.pipeline)
    };
    let info = cached.info.clone();
    let content_revision = cached.revision;
    let session_id = tracker.session_id;
    let sender = channel.sender.clone();
    let in_flight = Arc::clone(&channel.in_flight);

    tracker.mark_dispatched();
    channel.in_flight.fetch_add(1, Ordering::Relaxed);
    task.spawn_cpu(move || {
        let started = Instant::now();
        let mut lights = HashMap::new();
        let sources = rebuild_loaded_lighting(&snapshot, &info, &mut lights);
        let result = sender.send(LightingBuildResult {
            session_id,
            content_revision,
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
/// 替换时把内存回收成本重新带回渲染线程。
// Bevy 系统参数保持权威资源和客户端区块状态的访问边界显式。
#[allow(clippy::too_many_arguments)]
fn receive_lighting_results(
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

    let current_signature = current_world_signature(&world_state);
    if !lighting_result_is_current(&result, &tracker, &cached, &current_signature) {
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
        if !remesh.contains(&components.position) {
            continue;
        }
        runtime.bump_revision(components.position);
        if matches!(*state, ChunkState::GeneratingMesh | ChunkState::Rendered) {
            *state = ChunkState::StructureReady;
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

fn lighting_result_is_current(
    result: &LightingBuildResult,
    tracker: &LightingRebuildTracker,
    cached: &CachedLightInfo,
    current_signature: &[(IVec3, usize)],
) -> bool {
    result.session_id == tracker.session_id
        && result.content_revision == cached.revision
        && result.snapshot.signature() == current_signature
}

/// 离开世界时清除可重建光场，确保相同区块坐标的新会话不会复用旧快照。
fn clear_world_lighting(
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
        ..default()
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

fn current_world_signature(world: &WorldState) -> Vec<(IVec3, usize)> {
    let mut signature = world
        .chunks()
        .map(|(position, data)| (position, Arc::as_ptr(data) as usize))
        .collect::<Vec<_>>();
    signature.sort_by_key(|(position, _)| (position.x, position.y, position.z));
    signature
}

fn changed_light_chunks(
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

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/lighting/mod.rs"]
mod tests;
