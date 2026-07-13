use super::streaming::WorldStreamingConfig;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::context::ChunkGenContext;
use bevy::prelude::*;
use std::collections::HashSet;
use std::sync::{Mutex, mpsc};

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

#[derive(Resource, Default)]
pub struct PlayerChunkCache {
    pub last_chunk_pos: Option<IVec3>,
    pub last_streaming_config: Option<WorldStreamingConfig>,
    pub expected_chunks: HashSet<IVec3>,
    pub ordered_chunks: Vec<IVec3>,
}
