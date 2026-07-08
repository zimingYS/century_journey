use super::streaming::WorldStreamingConfig;
use crate::content::block::definition::RenderMode;
use crate::content::block::registry::BlockRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::context::ChunkGenContext;
use bevy::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, mpsc};

pub struct TerrainGenResult {
    pub chunk_pos: IVec3,
    pub chunk_data: ChunkData,
    pub gen_context: ChunkGenContext,
}

#[derive(Resource)]
pub struct TerrainGenChannel {
    pub sender: mpsc::Sender<TerrainGenResult>,
    pub receiver: Mutex<mpsc::Receiver<TerrainGenResult>>,
}

impl Default for TerrainGenChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

pub struct StructureGenResult {
    pub chunk_pos: IVec3,
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
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

pub struct MeshBuildResult {
    pub chunk_pos: IVec3,
    pub opaque: super::mesh_buffer::MeshBufferData,
    pub cutout: super::mesh_buffer::MeshBufferData,
    pub water: super::mesh_buffer::MeshBufferData,
}

#[derive(Resource)]
pub struct MeshBuildChannel {
    pub sender: mpsc::Sender<MeshBuildResult>,
    pub receiver: Mutex<mpsc::Receiver<MeshBuildResult>>,
}

impl Default for MeshBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

#[derive(Resource, Default)]
pub struct PlayerChunkCache {
    pub last_chunk_pos: Option<IVec3>,
    pub last_streaming_config: Option<WorldStreamingConfig>,
    pub expected_chunks: HashSet<IVec3>,
    pub ordered_chunks: Vec<IVec3>,
}

#[derive(Clone, Default)]
pub struct BlockInfoSnapshot {
    pub is_solid: Vec<bool>,
    pub render_modes: Vec<RenderMode>,
    pub texture_layers: Box<[u32]>,
    pub water_id: u16,
    pub total_layers: u32,
    pub max_id: u16,
}

impl BlockInfoSnapshot {
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

        for (&id, prop) in registry.iter_properties() {
            is_solid[id as usize] = prop.is_solid;
            render_modes[id as usize] = prop.render_mode;
        }

        let layer_count = (max_id as usize + 1) * 6;
        let mut flat_layers = vec![0u32; layer_count].into_boxed_slice();
        for (&(id, face), &layer) in registry.texture_layers_iter() {
            let idx = id as usize * 6 + face;
            if idx < layer_count {
                flat_layers[idx] = layer;
            }
        }

        Self {
            is_solid,
            render_modes,
            texture_layers: flat_layers,
            water_id,
            total_layers,
            max_id,
        }
    }

    #[inline]
    pub fn get_texture_layer(&self, voxel_id: u16, face_idx: usize) -> u32 {
        let idx = voxel_id as usize * 6 + face_idx;
        if idx < self.texture_layers.len() {
            self.texture_layers[idx]
        } else {
            0
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct CachedBlockInfo(pub BlockInfoSnapshot);

pub struct MeshBuildInput {
    pub chunk_pos: IVec3,
    pub current_data: Arc<ChunkData>,
    pub neighbors: [Option<Arc<ChunkData>>; 6],
    pub block_info: BlockInfoSnapshot,
}
