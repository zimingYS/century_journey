use super::streaming::WorldStreamingConfig;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::storage::PendingVoxel;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicUsize;
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

pub struct StructureGenResult {
    pub chunk_pos: IVec3,
    pub modified_chunks: Vec<(IVec3, ChunkData)>,
    pub pending_writes: HashMap<IVec3, Vec<PendingVoxel>>,
}

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

#[derive(Resource, Default)]
pub struct PlayerChunkCache {
    pub last_chunk_pos: Option<IVec3>,
    pub last_streaming_config: Option<WorldStreamingConfig>,
    pub expected_chunks: HashSet<IVec3>,
    pub ordered_chunks: Vec<IVec3>,
}
