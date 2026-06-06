use std::collections::HashSet;
use std::sync::{mpsc, Mutex};
use bevy::prelude::*;
use crate::voxel::properties::RenderMode;
use crate::voxel::registry::BlockRegistry;
use crate::world::chunk::ChunkData;
use crate::world::generation::context::ChunkGenContext;

/// 地形生成异步任务的结果
pub struct TerrainGenResult {
    pub chunk_pos: IVec3,
    pub chunk_data: ChunkData,
    pub gen_context: ChunkGenContext,
}

/// 地形生成通道资源
#[derive(Resource)]
pub struct TerrainGenChannel {
    pub sender: mpsc::Sender<TerrainGenResult>,
    pub receiver: Mutex<mpsc::Receiver<TerrainGenResult>>,
}

impl Default for TerrainGenChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver: Mutex::new(receiver) }
    }
}

/// 结构生成异步任务的结果
pub struct StructureGenResult {
    pub chunk_pos: IVec3,
    /// 所有被结构生成修改的区块（含邻居的跨区块写入）
    pub modified_chunks: Vec<(IVec3, ChunkData)>,
}

#[derive(Resource)]
pub struct StructureGenChannel {
    pub sender: mpsc::Sender<StructureGenResult>,
    pub receiver: Mutex<mpsc::Receiver<StructureGenResult>>,
}

impl Default for StructureGenChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver: Mutex::new(receiver) }
    }
}

/// Mesh 构建异步任务的结果
pub struct MeshBuildResult {
    pub chunk_pos: IVec3,
    pub opaque: super::mesh_buffer::MeshBufferData,
    pub cutout: super::mesh_buffer::MeshBufferData,
    pub water: super::mesh_buffer::MeshBufferData,
}

/// Mesh 构建通道资源
#[derive(Resource)]
pub struct MeshBuildChannel {
    pub sender: mpsc::Sender<MeshBuildResult>,
    pub receiver: Mutex<mpsc::Receiver<MeshBuildResult>>,
}

impl Default for MeshBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver: Mutex::new(receiver) }
    }
}

/// 玩家区块缓存
#[derive(Resource, Default)]
pub struct PlayerChunkCache {
    pub last_chunk_pos: Option<IVec3>,
    pub expected_chunks: HashSet<IVec3>,
}

/// 方块信息查找表
#[derive(Clone)]
pub struct BlockInfoSnapshot {
    pub is_solid: Vec<bool>,
    pub render_modes: Vec<RenderMode>,
    pub texture_layers: std::collections::HashMap<(u16, usize), u32>,
    pub water_id: u16,
    pub total_layers: u32,
}

impl BlockInfoSnapshot {
    pub fn from_registry(registry: &BlockRegistry) -> Self {
        let water_id = registry.get_id_by_identifier("century_journey:water").unwrap_or(0);
        let total_layers = registry.texture_layers.values().map(|&v| v + 1).max().unwrap_or(1);

        let max_id = registry.id_to_properties.keys().copied().max().unwrap_or(0);
        let mut is_solid = vec![false; (max_id + 1) as usize];
        let mut render_modes = vec![RenderMode::Opaque; (max_id + 1) as usize];

        for (&id, prop) in &registry.id_to_properties {
            is_solid[id as usize] = prop.is_solid;
            render_modes[id as usize] = prop.render_mode;
        }

        Self { is_solid, render_modes, texture_layers: registry.texture_layers.clone(), water_id, total_layers }
    }
}

/// 单个区块的 Mesh 构建输入快照
pub struct MeshBuildInput {
    pub chunk_pos: IVec3,
    pub current_data: ChunkData,
    pub neighbors: [Option<ChunkData>; 6],
    pub block_info: BlockInfoSnapshot,
}