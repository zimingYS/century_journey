/// 窗口常量
pub const WINDOW_WIDTH:u32 = 1280;
pub const WINDOW_HEIGHT:u32 = 720;
pub const WINDOW_TITLE:&'static str = "CenturyJourney";

// 世界相关常量定义
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

// 地形相关调整
pub const NOISE_SCALE:f64 = 0.005;
pub const MAP_HEIGHT_SCALE:f64 = 32.0;
pub const SEA_LEVEL:i32 = 64;

// 背包格子总数
pub const TOTAL_SLOTS:usize = 36;

// 贴图纹理大小
pub const TILE_SIZE: u32 = 16;
pub const TILE_SIZE_F32: f32 = TILE_SIZE as f32;