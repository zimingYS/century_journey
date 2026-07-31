//! 定义异步生成任务与主线程之间传递结果的有界通道资源。

use crate::game::world::TreeInstance;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::structure::pending_writes::PendingVoxel;
use crate::game::world::generation::terrain::context::ChunkGenContext;
use bevy::math::IVec3;
use bevy::prelude::{Entity, Resource};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};

/// 后台地形任务的成功载荷或不可替代的存档读取失败。
pub(in crate::game::world::generation) enum TerrainGenOutcome {
    /// 从待写内存快照或磁盘存档恢复的完整权威区块。
    Restored {
        /// 区块体素数据。
        chunk_data: Box<ChunkData>,
        /// 根坐标属于该区块的树实例。
        tree_instances: Vec<TreeInstance>,
    },
    /// 首次生成的基础地形，仍需进入结构生成阶段。
    Generated {
        /// 新生成的区块体素数据。
        chunk_data: Box<ChunkData>,
        /// 仅供后续结构生成使用的采样上下文。
        gen_context: ChunkGenContext,
    },
    /// 区块记录存在但无法安全读取，禁止退化为重新生成。
    LoadFailed(String),
}

/// 后台基础地形任务返回给主线程的带坐标结果。
pub struct TerrainGenResult {
    /// 任务对应的区块网格坐标。
    pub chunk_pos: IVec3,
    /// 派发任务时拥有此坐标的实体，用于拒绝卸载重载后的旧结果。
    pub request_entity: Entity,
    /// 成功数据或必须保留现场的加载失败。
    pub(super) outcome: TerrainGenOutcome,
}

/// 地形生成任务通道及当前飞行中任务计数。
#[derive(Resource)]
pub struct TerrainGenChannel {
    pub sender: mpsc::Sender<TerrainGenResult>,
    pub receiver: Mutex<mpsc::Receiver<TerrainGenResult>>,
    pub in_flight: Arc<AtomicUsize>,
}

impl Default for TerrainGenChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}

/// 后台结构任务返回的区块修改和跨区块延迟写入。
pub struct StructureGenResult {
    pub chunk_pos: IVec3,
    /// 派发结构任务时拥有此坐标的实体。
    pub request_entity: Entity,
    pub modified_chunks: Vec<(IVec3, ChunkData)>,
    pub pending_writes: HashMap<IVec3, Vec<PendingVoxel>>,
}

/// 结构生成任务通道及当前飞行中任务计数。
#[derive(Resource)]
pub struct StructureGenChannel {
    pub sender: mpsc::Sender<StructureGenResult>,
    pub receiver: Mutex<mpsc::Receiver<StructureGenResult>>,
    pub in_flight: Arc<AtomicUsize>,
}

impl Default for StructureGenChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}
