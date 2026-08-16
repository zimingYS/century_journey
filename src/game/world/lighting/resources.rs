//! 权威光照数据、重建追踪与后台任务通道等会话期资源。

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use bevy::prelude::*;

use crate::game::world::chunk::ChunkData;
use crate::game::world::lighting::chunk_light::{ChunkLight, LightCell};
use crate::game::world::lighting::rebuild::{
    BlockLightSource, GameLightInfo, LightingWorldSnapshot,
};
use crate::shared::voxel::CHUNK_SIZE;

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
    pub(super) chunk_snapshots: HashMap<IVec3, Arc<ChunkData>>,
}

impl WorldLighting {
    /// 判断指定光数组是否对应当前权威区块快照。
    pub fn is_chunk_light_current(&self, position: IVec3, data: &Arc<ChunkData>) -> bool {
        self.chunk_lights
            .get(&position)
            .is_some_and(|light| light.is_initialized())
            && self
                .chunk_snapshots
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
pub(super) struct CachedLightInfo {
    pub(super) info: GameLightInfo,
    pub(super) revision: u64,
}

/// 区块流送输入稳定后再做低频全局校正，交互和可见区块由局部任务先处理。
#[derive(Resource, Default)]
pub(super) struct LightingRebuildTracker {
    pub(super) observed_world_revision: u64,
    pub(super) observed_content_revision: u64,
    pub(super) stable_ticks: u8,
    pub(super) pending: bool,
    pub(super) urgent: bool,
    pub(super) task_defer_ticks: u16,
    pub(super) session_id: u64,
}

impl LightingRebuildTracker {
    /// 记录权威世界与内容修订号，驱动稳定窗口与紧急标记。
    pub(super) fn observe(&mut self, world_revision: u64, content_revision: u64) {
        if self.observed_world_revision != world_revision {
            self.observed_world_revision = world_revision;
            self.stable_ticks = 0;
            self.pending = true;
            self.task_defer_ticks = 0;
        } else if self.pending {
            self.stable_ticks = self.stable_ticks.saturating_add(1);
        }

        if self.observed_content_revision != content_revision {
            self.observed_content_revision = content_revision;
            self.pending = true;
            self.urgent = true;
            self.task_defer_ticks = 0;
        }
    }

    /// 判断当前是否满足派发全局重建任务的条件。
    pub(super) fn ready_to_dispatch(&self, in_flight: usize) -> bool {
        in_flight == 0
            && self.pending
            && (self.urgent || self.stable_ticks >= WORLD_REBUILD_STABLE_TICKS)
    }

    /// 判断任务池积压是否已经达到全局校正的最大延期界限。
    pub(super) fn task_backlog_expired(&self) -> bool {
        self.task_defer_ticks >= WORLD_REBUILD_MAX_TASK_DEFER_TICKS
    }

    /// 记录任务池积压，并在达到上限后允许一次全局校正进入共享执行器。
    pub(super) fn should_defer_for_task_backlog(&mut self, pending_tasks: usize) -> bool {
        if pending_tasks == 0 {
            self.task_defer_ticks = 0;
            return false;
        }
        self.task_defer_ticks = self.task_defer_ticks.saturating_add(1);
        !self.task_backlog_expired()
    }

    /// 在派发后清空待重建与紧急标记。
    pub(super) fn mark_dispatched(&mut self) {
        self.pending = false;
        self.urgent = false;
        self.stable_ticks = 0;
        self.task_defer_ticks = 0;
    }
}

/// 后台重建完成后一次性交还给固定步提交系统的数据。
pub(super) struct LightingBuildResult {
    pub(super) session_id: u64,
    pub(super) content_revision: u64,
    pub(super) world_revision: u64,
    pub(super) snapshot: LightingWorldSnapshot,
    pub(super) lights: HashMap<IVec3, ChunkLight>,
    pub(super) sources: Vec<BlockLightSource>,
    pub(super) elapsed: Duration,
}

/// 限制全窗口光照最多只有一个后台任务，并负责跨线程移交结果。
#[derive(Resource)]
pub(super) struct LightingBuildChannel {
    pub(super) sender: mpsc::Sender<LightingBuildResult>,
    pub(super) receiver: Mutex<mpsc::Receiver<LightingBuildResult>>,
    pub(super) in_flight: Arc<AtomicUsize>,
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
pub(super) const WORLD_REBUILD_STABLE_TICKS: u8 = 20;
/// 任务池持续繁忙时，全局校正最多再延迟约四秒，避免远区块永久没有最终光场。
pub(super) const WORLD_REBUILD_MAX_TASK_DEFER_TICKS: u16 = 80;
