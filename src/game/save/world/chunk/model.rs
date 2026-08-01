//! 定义磁盘区块记录以及运行时区块之间的转换。

use crate::game::world::TreeInstance;
use crate::game::world::chunk::ChunkData;
use crate::game::world::state::WorldChunkSnapshot;
use bevy::math::IVec3;
use serde::{Deserialize, Serialize};

/// Region 文件每条坐标轴容纳的区块数。
///
/// 该值参与区块坐标映射和位图长度，修改时必须升级并迁移存档格式。
pub(super) const REGION_SIZE: i32 = 32;

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

/// 单个区块的持久化数据。
///
/// 树实例只随树根所属区块保存，跨入相邻区块的树干和树叶仍由各自体素快照持有。
#[derive(Clone, Debug)]
pub struct SavedChunk {
    /// 区块世界坐标。
    pub position: IVec3,
    /// 区块方块数据。
    pub data: ChunkData,
    /// 根坐标属于该区块的有序树实例。
    pub tree_instances: Vec<TreeInstance>,
    /// 区块最后修改时间（自存档创建以来的秒数）。
    pub modified_time: f64,
}

impl SavedChunk {
    /// 从世界层的聚合快照创建存档记录，保持体素与树实例处于同一时刻。
    pub(in crate::game) fn from_world_snapshot(
        position: IVec3,
        snapshot: WorldChunkSnapshot,
        modified_time: f64,
    ) -> Self {
        Self {
            position,
            data: snapshot.data.as_ref().clone(),
            tree_instances: snapshot.tree_instances,
            modified_time,
        }
    }

    /// 校验树根归属、唯一性和时间戳，阻止损坏载荷进入权威世界。
    pub(super) fn validate(&self) -> Result<(), String> {
        if !self.modified_time.is_finite() {
            return Err("区块修改时间必须是有限值".into());
        }
        let mut roots = std::collections::HashSet::new();
        for instance in &self.tree_instances {
            if instance.owner_chunk() != self.position {
                return Err(format!(
                    "树根 {:?} 不属于存档区块 {:?}",
                    instance.root(),
                    self.position
                ));
            }
            if !roots.insert(instance.root()) {
                return Err(format!("存档区块包含重复树根 {:?}", instance.root()));
            }
        }
        Ok(())
    }
}

/// 将区块世界坐标转换为所属 Region 坐标。
///
/// 使用欧几里得除法，确保负坐标区块与正坐标遵循相同的 Region 划分规则。
#[inline]
pub fn chunk_to_region_pos(chunk_pos: IVec3) -> IVec3 {
    IVec3::new(
        chunk_pos.x.div_euclid(REGION_SIZE),
        chunk_pos.y.div_euclid(REGION_SIZE),
        chunk_pos.z.div_euclid(REGION_SIZE),
    )
}

/// 计算区块在所属 Region 内的三维局部索引。
///
/// 使用欧几里得余数，保证负坐标不会产生越界的局部位置。
#[inline]
pub fn chunk_local_index(chunk_pos: IVec3) -> (usize, usize, usize) {
    let local = |value: i32| value.rem_euclid(REGION_SIZE) as usize;
    (local(chunk_pos.x), local(chunk_pos.y), local(chunk_pos.z))
}

/// 按 Region 文件的 Y-Z-X 顺序将三维局部索引展平为位图索引。
#[inline]
pub fn local_index_to_flat(local_x: usize, local_y: usize, local_z: usize) -> usize {
    let region_size = REGION_SIZE as usize;
    local_y * region_size * region_size + local_z * region_size + local_x
}
