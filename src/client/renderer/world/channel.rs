//! 定义后台区块网格任务向渲染主线程回传结果的通道。

use super::mesh_buffer::MeshBufferData;
use crate::content::block::definition::RenderMode;
use crate::content::block::model::BlockModel;
use crate::content::block::registry::BlockRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::lighting::chunk_light::ChunkLight;
use bevy::prelude::*;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};

/// 后台网格任务返回的三种渲染通道。
pub struct MeshBuildResult {
    pub chunk_pos: IVec3,
    /// 派发任务时对应的区块实体，用于拒绝卸载后同坐标新实体收到旧结果。
    pub request_entity: Entity,
    /// 单调递增的区块网格请求编号，用于拒绝同一实体上的乱序旧结果。
    pub request_id: u64,
    pub opaque: MeshBufferData,
    pub cutout: MeshBufferData,
    pub water: MeshBufferData,
}

/// Client 层的区块网格任务通道。
#[derive(Resource)]
pub struct MeshBuildChannel {
    pub sender: mpsc::Sender<MeshBuildResult>,
    pub receiver: Mutex<mpsc::Receiver<MeshBuildResult>>,
    pub in_flight: Arc<AtomicUsize>,
}

impl Default for MeshBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}

/// 后台区块网格任务需要识别的方块几何类型。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BlockMeshKind {
    #[default]
    Cube,
    Cross,
}

/// 网格任务使用的方块渲染属性快照，避免后台线程访问 ECS 资源。
#[derive(Clone, Default)]
pub struct BlockInfoSnapshot {
    pub is_solid: Vec<bool>,
    pub render_modes: Vec<RenderMode>,
    mesh_kinds: Vec<BlockMeshKind>,
    pub model_random_rotations: Vec<bool>,
    pub texture_layers: Box<[u32]>,
    pub water_id: u16,
    pub total_layers: u32,
    pub max_id: u16,
}

impl BlockInfoSnapshot {
    /// 从内容注册表复制后台网格任务所需的最小只读属性。
    pub fn from_registry(registry: &BlockRegistry) -> Self {
        let water_id = registry
            .get_id_by_identifier("century_journey:water")
            .unwrap_or(0);
        let total_layers = registry.total_layer_count().max(1) as u32;
        let max_id = registry
            .iter_properties()
            .map(|(&id, _)| id)
            .max()
            .unwrap_or(0);
        let mut is_solid = vec![false; (max_id + 1) as usize];
        let mut render_modes = vec![RenderMode::Opaque; (max_id + 1) as usize];
        let mut mesh_kinds = vec![BlockMeshKind::Cube; (max_id + 1) as usize];
        let mut model_random_rotations = vec![false; (max_id + 1) as usize];

        for (&id, property) in registry.iter_properties() {
            is_solid[id as usize] = property.is_solid;
            render_modes[id as usize] = property.render_mode;
            mesh_kinds[id as usize] = match property.model.model {
                BlockModel::Cross => BlockMeshKind::Cross,
                _ => BlockMeshKind::Cube,
            };
            model_random_rotations[id as usize] = property.model.random_rotation;
        }

        let layer_count = (max_id as usize + 1) * 6;
        let mut texture_layers = vec![0u32; layer_count].into_boxed_slice();
        for (&(id, face), &layer) in registry.texture_layers_iter() {
            let index = id as usize * 6 + face;
            if index < layer_count {
                texture_layers[index] = layer;
            }
        }

        Self {
            is_solid,
            render_modes,
            mesh_kinds,
            model_random_rotations,
            texture_layers,
            water_id,
            total_layers,
            max_id,
        }
    }

    #[inline]
    /// 返回方块指定面的图集层，越界编号回退到首层。
    pub fn get_texture_layer(&self, voxel_id: u16, face_index: usize) -> u32 {
        let index = voxel_id as usize * 6 + face_index;
        self.texture_layers.get(index).copied().unwrap_or(0)
    }

    /// 判断方块是否应由独立的十字平面网格处理。
    #[inline]
    pub fn is_cross_model(&self, voxel_id: u16) -> bool {
        self.mesh_kinds
            .get(voxel_id as usize)
            .is_some_and(|kind| *kind == BlockMeshKind::Cross)
    }

    /// 判断方块模型是否应使用基于世界坐标的确定性旋转。
    #[inline]
    pub fn uses_random_model_rotation(&self, voxel_id: u16) -> bool {
        self.model_random_rotations
            .get(voxel_id as usize)
            .copied()
            .unwrap_or(false)
    }
}

/// ECS 中缓存的方块渲染属性快照。
#[derive(Resource, Default, Clone)]
pub struct CachedBlockInfo(pub BlockInfoSnapshot);

/// 发送到后台网格任务的只读区块数据。
pub struct MeshBuildInput {
    pub chunk_pos: IVec3,
    /// 派发任务时对应的区块实体。
    pub request_entity: Entity,
    /// 本次区块网格请求的唯一编号。
    pub request_id: u64,
    pub current_data: Arc<ChunkData>,
    pub neighbors: [Option<Arc<ChunkData>>; 6],
    pub block_info: BlockInfoSnapshot,
    /// 本区块光级快照（异步线程内只读，不允许访问 ECS）。
    pub light: Option<Arc<ChunkLight>>,
    /// 邻居光级快照（跨区块边界采样用）。
    pub neighbor_lights: [Option<Arc<ChunkLight>>; 6],
}
