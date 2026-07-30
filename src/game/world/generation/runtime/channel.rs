//! 定义异步生成任务与主线程之间传递结果的有界通道资源。

use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::structure::pending_writes::PendingVoxel;
use crate::game::world::generation::terrain::context::ChunkGenContext;
use bevy::math::IVec3;
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};

/// 后台基础地形任务返回给主线程的结果。
pub struct TerrainGenResult {
    pub chunk_pos: IVec3,
    pub chunk_data: ChunkData,
    pub gen_context: ChunkGenContext,
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
