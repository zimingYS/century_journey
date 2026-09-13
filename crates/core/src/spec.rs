//! 此处用于存放一些常量

// --- Chunk & World ---
/// 区块大小
pub const CHUNK_SIZE: i32 = 16;

/// 区块切片尺寸
pub const SECTION_SIZE: i32 = 16;

/// 世界最低Y坐标
pub const WORLD_BOTTOM: i32 = -256;

/// 世界最高Y坐标
pub const WORLD_TOP: i32 = 512;

/// 海平面
pub const SEA_LEVEL: i32 = 128;

/// 垂直最大容纳切片数
pub const SECTION_COUNT: i32 = (WORLD_TOP - WORLD_BOTTOM) / SECTION_SIZE;

/// 最低切片索引
pub const SECTION_MIN_Y: i32 = WORLD_BOTTOM / SECTION_SIZE;

/// 最高切片索引
pub const SECTION_INDEX_LIMIT: i32 = WORLD_TOP / SECTION_SIZE;
