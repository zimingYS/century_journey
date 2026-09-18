use cj_core::pos::{BlockPos, SectionPos};
use cj_core::spec::{CHUNK_SIZE, SECTION_SIZE};
use cj_core::voxel::BlockId;

/// 构建检查快照格子数
pub const SNAPSHOT_SIZE: usize = 18;

/// 构造三维检查快照
pub const SNAPSHOT_VOLUME: usize = SNAPSHOT_SIZE * SNAPSHOT_SIZE * SNAPSHOT_SIZE;

/// 使用世界坐标查询方块
pub trait BlockView {
    fn block(&self, pos: BlockPos) -> BlockId;
}

/// 单个区块切片快照，缓存对应数据
pub struct SectionSnapshot {
    /// 对应快照切片
    section: SectionPos,
    /// 方块数据
    blocks: Box<[BlockId; SNAPSHOT_VOLUME]>,
}

impl SectionSnapshot {
    /// 构建全空气快照
    pub fn new(section: SectionPos) -> Self {
        SectionSnapshot {
            section,
            blocks: Box::new([BlockId::AIR; SNAPSHOT_VOLUME]),
        }
    }

    /// 获取对应区块切片
    #[inline]
    pub fn section(&self) -> SectionPos {
        self.section
    }

    // 将世界坐标转换为快照线性索引
    #[inline]
    fn index_of(&self, pos: BlockPos) -> Option<usize> {
        let origin_x = self.section.chunk.x * CHUNK_SIZE;
        let origin_y = self.section.y * SECTION_SIZE;
        let origin_z = self.section.chunk.z * CHUNK_SIZE;

        let x = pos.x - origin_x + 1;
        let y = pos.y - origin_y + 1;
        let z = pos.z - origin_z + 1;

        if !(0..SNAPSHOT_SIZE as i32).contains(&x)
            || !(0..SNAPSHOT_SIZE as i32).contains(&y)
            || !(0..SNAPSHOT_SIZE as i32).contains(&z)
        {
            return None;
        }

        let x = x as usize;
        let y = y as usize;
        let z = z as usize;

        Some(x + z * SNAPSHOT_SIZE + y * SNAPSHOT_SIZE * SNAPSHOT_SIZE)
    }

    /// 按世界坐标写入方块。
    #[inline]
    pub fn set(&mut self, pos: BlockPos, block: BlockId) {
        if let Some(index) = self.index_of(pos) {
            self.blocks[index] = block;
        }
    }

    /// 按世界坐标查询方块。
    #[inline]
    pub fn get(&self, pos: BlockPos) -> BlockId {
        self.index_of(pos)
            .map(|index| self.blocks[index])
            .unwrap_or(BlockId::AIR)
    }
}

impl BlockView for SectionSnapshot {
    #[inline]
    fn block(&self, pos: BlockPos) -> BlockId {
        self.get(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cj_core::pos::ChunkPos;

    fn test_section() -> SectionPos {
        SectionPos::from_index(ChunkPos { x: 0, z: 0 }, 16)
    }

    fn offset_code(ox: i32, oy: i32, oz: i32) -> BlockId {
        BlockId::from_raw(((ox + 1) * 324 + (oy + 1) * 18 + (oz + 1)) as u16)
    }

    #[test]
    fn snapshot_keeps_its_section() {
        let section = test_section();
        assert_eq!(
            SectionSnapshot::new(section).section(),
            section,
            "快照应保留构造时传入的 section"
        );
    }

    #[test]
    fn snapshot_addresses_all_padding_offsets() {
        let mut snapshot = SectionSnapshot::new(test_section());

        for oy in -1..17 {
            for oz in -1..17 {
                for ox in -1..17 {
                    snapshot.set(BlockPos::new(ox, oy, oz), offset_code(ox, oy, oz));
                }
            }
        }

        for oy in -1..17 {
            for oz in -1..17 {
                for ox in -1..17 {
                    assert_eq!(
                        snapshot.get(BlockPos::new(ox, oy, oz)),
                        offset_code(ox, oy, oz),
                        "偏移 ({ox},{oy},{oz}) 读写不一致 —— pad 圈必须可寻址且不串轴"
                    );
                }
            }
        }
    }

    #[test]
    fn snapshot_rejects_out_of_range() {
        let mut snapshot = SectionSnapshot::new(test_section());
        snapshot.set(BlockPos::new(0, 0, 0), BlockId::STONE);

        for pos in [
            BlockPos::new(17, 0, 0),
            BlockPos::new(0, 17, 0),
            BlockPos::new(0, 0, 17),
            BlockPos::new(-2, 0, 0),
            BlockPos::new(0, -2, 0),
            BlockPos::new(0, 0, -2),
        ] {
            assert_eq!(
                snapshot.get(pos),
                BlockId::AIR,
                "{pos:?} 超出 pad 范围，应返回 AIR"
            );
        }
    }

    #[test]
    fn snapshot_ignores_out_of_range_write() {
        let mut snapshot = SectionSnapshot::new(test_section());
        snapshot.set(BlockPos::new(0, 0, 0), BlockId::STONE);
        snapshot.set(BlockPos::new(64, 64, 64), BlockId::WATER);

        assert_eq!(
            snapshot.get(BlockPos::new(0, 0, 0)),
            BlockId::STONE,
            "越界写入不应破坏已有数据"
        );
    }

    #[test]
    fn snapshot_forwards_block_view() {
        let mut snapshot = SectionSnapshot::new(test_section());
        snapshot.set(BlockPos::new(3, 4, 5), BlockId::GRASS);

        let view: &dyn BlockView = &snapshot;

        assert_eq!(
            view.block(BlockPos::new(3, 4, 5)),
            BlockId::GRASS,
            "BlockView 实现应转发到快照查询"
        );
        assert!(
            view.block(BlockPos::new(3, 4, 6)).is_air(),
            "未写入的格子应视为空气"
        );
    }
}
