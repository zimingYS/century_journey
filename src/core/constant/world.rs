/*
    区块与地形生成
*/
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
pub const NOISE_SCALE: f64 = 0.005;
pub const MAP_HEIGHT_SCALE: f64 = 32.0;
pub const SEA_LEVEL: i32 = 64;
pub const RENDER_DISTANCE: i32 = 8;
pub const DATA_DISTANCE: i32 = RENDER_DISTANCE + 1;
/// 每帧最多生成区块实体数
pub const MAX_SPAWN_PER_FRAME: u32 = 4;
/// 每帧最多派发地形任务数
pub const MAX_TERRAIN_TASKS_PER_FRAME: u32 = 4;
/// 每帧最多派发 Mesh 构建任务数
pub const MAX_MESH_TASKS_PER_FRAME: u32 = 16;
/// 接收结果每帧最多处理数量
pub const MAX_TERRAIN_RECEIVE_PER_FRAME: usize = 4;
pub const MAX_MESH_RECEIVE_PER_FRAME: usize = 16;
/// 结构生成每帧时间预算 (ms)
pub const STRUCTURE_BUDGET_MS: f64 = 2.0;
/// 面分组为空时的哨兵值
pub const FACE_NONE: u32 = u32::MAX;
/// 每帧最多派发结构生成任务数
pub const MAX_STRUCTURE_TASKS_PER_FRAME: u32 = 4;
/// 每帧最多接收结构生成结果数
pub const MAX_STRUCTURE_RECEIVE_PER_FRAME: usize = 4;


/*
    保存读取世界
*/
/// Region 文件中每维的区块数量 (32×32 = 1024 个区块)
pub const REGION_SIZE: i32 = 32;

/// 默认世界存档名称
pub const DEFAULT_WORLD_NAME: &str = "NEW WORLD";

/// 存档根目录名
pub const SAVE_DIR_NAME: &str = "saves";

/// Level 元数据文件名
pub const LEVEL_FILE_NAME: &str = "level.dat";

/// Region 子目录名
pub const REGION_DIR_NAME: &str = "regions";

/// Region 文件名前缀
pub const REGION_FILE_PREFIX: &str = "r";

/// 自动保存间隔（秒）
pub const AUTO_SAVE_INTERVAL_SECS: f64 = 60.0;

/// 单个区块原始数据大小 (4096 * 2bytes)
pub const CHUNK_RAW_SIZE: usize = 4096 * 2;

// 每帧最多处理 N 个区块
pub const MAX_SAVE_PER_FRAME: usize = 4;

/*
    气候与生物群系
*/
/// 温度噪声缩放
pub const TEMPERATURE_NOISE_SCALE: f64 = 0.001;
/// 湿度噪声缩放
pub const HUMIDITY_NOISE_SCALE: f64 = 0.0015;
/// 生物群系过渡平滑半径（区块数）
pub const BIOME_BLEND_RADIUS: i32 = 2;
/// 结构生成种子偏移
pub const STRUCTURE_SEED_OFFSET: u32 = 7919;
/// 一天对应的现实秒数
pub const SECONDS_PER_DAY: f32 = 1440.0;
/// 一天对应的游戏小时数
pub const HOURS_PER_DAY: f32 = 1440.0;

/// 全局噪声缩放 — 所有群系共用，保证跨区块连续
pub const GLOBAL_TERRAIN_SCALE: f64 = 0.005;
pub const GLOBAL_DETAIL_SCALE: f64 = 0.02;
pub const GLOBAL_ROUGHNESS_SCALE: f64 = 0.01;

/// 气候稀疏采样分辨率 — 温度/湿度 scale≈0.001，极低频率，大幅降采样无损
pub const CLIMATE_SPARSE: usize = 4;

/// 地形噪声稀疏采样分辨率 — scale 0.005~0.02，stride≈2 足够精细
pub const TERRAIN_SPARSE: usize = 9;