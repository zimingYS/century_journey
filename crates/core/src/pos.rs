//! 坐标类型

/// 一个 chunk 的边长（方块数）
pub const CHUNK_SIZE:i32 = 16;

/// 世界坐标（方块整数坐标）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPos{
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Chunk 坐标（xz 平面）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl WorldPos {
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(CHUNK_SIZE),
            z: self.z.div_euclid(CHUNK_SIZE),
        }
    }
}