use crate::game::world::chunk::ChunkData;
use bevy::prelude::IVec3;
use serde::{Deserialize, Serialize};

/// 世界数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    /// 世界种子
    pub seed: u64,
    /// 出生地坐标
    pub spawn_position: [f32; 3],
    /// 游戏时间
    pub time_of_day: f32,
    /// 区块方块 ID
    pub block_id_map: Vec<(u16, String)>,
    /// 存档版本号
    pub version: f32,
}

impl LevelData {
    pub const CURRENT_VERSION: f32 = 0.1;
}

/// Region 文件整体结构
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegionFile {
    /// 文件头
    pub header: RegionHeader,
    /// 所有存在区块的压缩数据
    pub chunks: Vec<Vec<u8>>,
}

/// 定位每个区块数据的偏移量和长度
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegionHeader {
    /// 区块存在标记位图
    pub chunk_present: Vec<u8>,
    /// 每个存在区块在文件中的字节偏移
    pub chunk_offsets: Vec<u64>,
    /// 每个存在区块的压缩数据长度
    pub chunk_lengths: Vec<u32>,
}

/// 单个区块的持久化数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedChunk {
    /// 区块世界坐标
    pub position: IVec3,
    /// 区块方块数据
    pub data: ChunkData,
    /// 区块最后修改时间（自存档创建以来的秒数）
    pub modified_time: f64,
}

/// 计算区块坐标对应的 Region 坐标
#[inline]
pub fn chunk_to_region_pos(chunk_pos: IVec3) -> IVec3 {
    use crate::content::constant::world::REGION_SIZE;
    IVec3::new(
        chunk_pos.x.div_euclid(REGION_SIZE),
        chunk_pos.y.div_euclid(REGION_SIZE),
        chunk_pos.z.div_euclid(REGION_SIZE),
    )
}

/// 计算区块在 Region 内的局部索引
#[inline]
pub fn chunk_local_index(chunk_pos: IVec3) -> (usize, usize, usize) {
    use crate::content::constant::world::REGION_SIZE;
    let local = |v: i32| v.rem_euclid(REGION_SIZE) as usize;
    (local(chunk_pos.x), local(chunk_pos.y), local(chunk_pos.z))
}

/// 三维局部索引转一维索引
#[inline]
pub fn local_index_to_flat(lx: usize, ly: usize, lz: usize) -> usize {
    use crate::content::constant::world::REGION_SIZE;
    let rs = REGION_SIZE as usize;
    ly * rs * rs + lz * rs + lx
}
