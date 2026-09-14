//! 世界规格常量

// --- Chunk & World ---
/// 区块水平尺寸
pub const CHUNK_SIZE: i32 = 16;

/// 切片尺寸
pub const SECTION_SIZE: i32 = 16;

/// 世界最低 Y
pub const WORLD_BOTTOM: i32 = -256;

/// 世界最高 Y（不含）
pub const WORLD_TOP: i32 = 512;

/// 海平面 Y
pub const SEA_LEVEL: i32 = 128;

/// 竖直切片数
pub const SECTION_COUNT: i32 = (WORLD_TOP - WORLD_BOTTOM) / SECTION_SIZE;
pub const SECTION_COUNT_USIZE: usize = SECTION_COUNT as usize;

/// 切片索引下界
pub const SECTION_MIN_Y: i32 = WORLD_BOTTOM / SECTION_SIZE;

/// 切片索引上界（不含）
pub const SECTION_INDEX_LIMIT: i32 = WORLD_TOP / SECTION_SIZE;
