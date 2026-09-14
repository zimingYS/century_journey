//! 坐标类型

use crate::spec::{CHUNK_SIZE, SECTION_COUNT, SECTION_COUNT_USIZE, SECTION_MIN_Y, SECTION_SIZE};

/// 世界方块坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// 切片内部坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionLocalPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

/// Chunk 在 XZ 平面的坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Section 坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionPos {
    pub chunk: ChunkPos,
    pub y: i32,
}

/// 世界列坐标（竖直方块柱）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnPos {
    pub x: i32,
    pub z: i32,
}

impl BlockPos {
    /// 新建坐标
    pub fn new(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    /// 所属 Chunk 坐标
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(CHUNK_SIZE),
            z: self.z.div_euclid(CHUNK_SIZE),
        }
    }

    /// 所属 Section 坐标
    pub fn section_pos(&self) -> SectionPos {
        SectionPos {
            chunk: self.chunk_pos(),
            y: self.y.div_euclid(SECTION_SIZE),
        }
    }

    /// 在所属 Section 内的局部坐标
    pub fn section_local(&self) -> SectionLocalPos {
        SectionLocalPos {
            x: self.x.rem_euclid(CHUNK_SIZE) as u8,
            y: self.y.rem_euclid(SECTION_SIZE) as u8,
            z: self.z.rem_euclid(CHUNK_SIZE) as u8,
        }
    }
}

impl SectionPos {
    /// 转为整数索引
    pub fn index(&self) -> usize {
        let index = self.y - SECTION_MIN_Y;

        debug_assert!(
            (0..SECTION_COUNT).contains(&index),
            "section y out of range: {}",
            self.y
        );

        index as usize
    }

    /// 由整数索引构造
    pub fn from_index(chunk: ChunkPos, index: usize) -> Self {
        debug_assert!(
            index < SECTION_COUNT_USIZE,
            "section index out of range: {index}"
        );

        Self {
            chunk,
            y: index as i32 + SECTION_MIN_Y,
        }
    }

    /// 局部位对应的世界方块坐标
    pub fn block_at(&self, local: SectionLocalPos) -> BlockPos {
        BlockPos {
            x: self.chunk.x * CHUNK_SIZE + local.x as i32,
            y: self.y * SECTION_SIZE + local.y as i32,
            z: self.chunk.z * CHUNK_SIZE + local.z as i32,
        }
    }

    /// Section 底面世界 Y
    pub fn block_y_start(&self) -> i32 {
        self.y * SECTION_SIZE
    }

    /// 世界 Y 在本 Section 内的局部 Y
    pub fn to_section_y(&self, block_y: i32) -> u8 {
        block_y.rem_euclid(SECTION_SIZE) as u8
    }

    /// 世界方块在本 Section 内的局部坐标
    pub fn local_from(&self, block: BlockPos) -> SectionLocalPos {
        debug_assert_eq!(
            block.section_pos(),
            *self,
            "block {:?} does not belong to section {:?}",
            block,
            self,
        );

        SectionLocalPos {
            x: block.x.rem_euclid(CHUNK_SIZE) as u8,
            y: self.to_section_y(block.y),
            z: block.z.rem_euclid(CHUNK_SIZE) as u8,
        }
    }
}

impl ChunkPos {
    /// 由世界方块坐标计算
    pub fn from_block(block_pos: BlockPos) -> ChunkPos {
        block_pos.chunk_pos()
    }

    /// Chunk 的世界方块原点
    pub fn origin(&self) -> BlockPos {
        BlockPos {
            x: self.x * CHUNK_SIZE,
            y: 0,
            z: self.z * CHUNK_SIZE,
        }
    }

    /// 水平四邻偏移
    pub const NEIGHBORS_4: [(i32, i32); 4] = [
        (0, -1), // north
        (1, 0),  // east
        (0, 1),  // south
        (-1, 0), // west
    ];

    /// 迭代四个水平相邻 Chunk
    pub fn neighbors_4(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        Self::NEIGHBORS_4.iter().map(move |&(dx, dz)| ChunkPos {
            x: self.x + dx,
            z: self.z + dz,
        })
    }

    /// 与其他 Chunk 的 XZ 切比雪夫距离（视距 / LOD）
    pub fn chebyshev_to(&self, other: ChunkPos) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dz = (self.z - other.z).unsigned_abs();
        dx.max(dz)
    }
}

impl SectionLocalPos {
    /// 转为线性数组下标
    pub fn index(&self) -> usize {
        let size = SECTION_SIZE as usize;
        let x = self.x as usize;
        let y = self.y as usize;
        let z = self.z as usize;
        x + z * size + y * size * size
    }
}

impl ColumnPos {
    /// 新建列坐标
    #[inline]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// 所属 Chunk 坐标
    #[inline]
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(CHUNK_SIZE),
            z: self.z.div_euclid(CHUNK_SIZE),
        }
    }

    /// Chunk 内局部 X
    #[inline]
    pub fn local_x(&self) -> u8 {
        self.x.rem_euclid(CHUNK_SIZE) as u8
    }

    /// Chunk 内局部 Z
    #[inline]
    pub fn local_z(&self) -> u8 {
        self.z.rem_euclid(CHUNK_SIZE) as u8
    }

    /// 由 Chunk 坐标与局部列坐标构造
    #[inline]
    pub fn from_chunk_local(chunk: ChunkPos, lx: u8, lz: u8) -> Self {
        debug_assert!(
            (lx as i32) < CHUNK_SIZE,
            "local x out of range: {lx} >= {CHUNK_SIZE}"
        );
        debug_assert!(
            (lz as i32) < CHUNK_SIZE,
            "local z out of range: {lz} >= {CHUNK_SIZE}"
        );

        Self {
            x: chunk.x * CHUNK_SIZE + lx as i32,
            z: chunk.z * CHUNK_SIZE + lz as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::SEA_LEVEL;

    // --- BlockPos 分块换算 ---

    #[test]
    fn block_to_chunk_positive() {
        assert_eq!(BlockPos::new(0, 0, 0).chunk_pos(), ChunkPos { x: 0, z: 0 });
        assert_eq!(
            BlockPos::new(15, 0, 15).chunk_pos(),
            ChunkPos { x: 0, z: 0 }
        );
        assert_eq!(
            BlockPos::new(16, 0, 16).chunk_pos(),
            ChunkPos { x: 1, z: 1 }
        );
        assert_eq!(
            BlockPos::new(31, 0, 31).chunk_pos(),
            ChunkPos { x: 1, z: 1 }
        );
    }

    #[test]
    fn block_to_chunk_negative() {
        assert_eq!(
            BlockPos::new(-1, 0, -1).chunk_pos(),
            ChunkPos { x: -1, z: -1 }
        );
        assert_eq!(
            BlockPos::new(-16, 0, -16).chunk_pos(),
            ChunkPos { x: -1, z: -1 }
        );
        assert_eq!(
            BlockPos::new(-17, 0, -17).chunk_pos(),
            ChunkPos { x: -2, z: -2 }
        );
    }

    #[test]
    fn section_index_round_trip() {
        for index in 0..SECTION_COUNT_USIZE {
            let section = SectionPos::from_index(ChunkPos { x: 3, z: -4 }, index);
            assert_eq!(section.index(), index, "round trip failed at {index}");
            assert_eq!(section.chunk, ChunkPos { x: 3, z: -4 });
        }
    }

    #[test]
    fn section_index_boundaries() {
        let chunk = ChunkPos { x: 0, z: 0 };

        assert_eq!(SectionPos { chunk, y: -16 }.index(), 0);
        assert_eq!(SectionPos { chunk, y: 31 }.index(), SECTION_COUNT_USIZE - 1);

        let sea_index = (SEA_LEVEL / SECTION_SIZE - SECTION_MIN_Y) as usize;
        assert_eq!(
            SectionPos {
                chunk,
                y: SEA_LEVEL / SECTION_SIZE
            }
            .index(),
            sea_index
        );
    }

    #[test]
    fn section_local_negative_block() {
        let local = BlockPos::new(-1, -1, -1).section_local();
        assert_eq!((local.x, local.y, local.z), (15, 15, 15));

        let local = BlockPos::new(-16, -16, -16).section_local();
        assert_eq!((local.x, local.y, local.z), (0, 0, 0));
    }

    #[test]
    fn section_local_index_is_in_range() {
        let local = SectionLocalPos {
            x: 15,
            y: 15,
            z: 15,
        };
        assert_eq!(local.index(), 15 + 15 * 16 + 15 * 256);
        assert!(local.index() < 4096);
    }

    #[test]
    fn section_pos_block_at_round_trip() {
        let section = SectionPos {
            chunk: ChunkPos { x: -2, z: 5 },
            y: -3,
        };
        let local = SectionLocalPos { x: 7, y: 9, z: 12 };
        let block = section.block_at(local);

        assert_eq!(block.section_pos(), section);
        assert_eq!(section.local_from(block), local);
    }

    // --- ColumnPos ---

    #[test]
    fn column_chunk_pos_negative() {
        assert_eq!(
            ColumnPos::new(-1, -1).chunk_pos(),
            ChunkPos { x: -1, z: -1 }
        );
        assert_eq!(
            ColumnPos::new(-16, -16).chunk_pos(),
            ChunkPos { x: -1, z: -1 }
        );
        assert_eq!(
            ColumnPos::new(-17, -17).chunk_pos(),
            ChunkPos { x: -2, z: -2 }
        );
    }

    #[test]
    fn column_local_negative() {
        assert_eq!(
            (
                ColumnPos::new(-1, -1).local_x(),
                ColumnPos::new(-1, -1).local_z()
            ),
            (15, 15)
        );
        assert_eq!(
            (
                ColumnPos::new(-16, -16).local_x(),
                ColumnPos::new(-16, -16).local_z()
            ),
            (0, 0)
        );
    }

    #[test]
    fn column_from_chunk_local_round_trip() {
        let chunk = ChunkPos { x: -3, z: 2 };

        for lx in 0..CHUNK_SIZE as u8 {
            for lz in 0..CHUNK_SIZE as u8 {
                let column = ColumnPos::from_chunk_local(chunk, lx, lz);
                assert_eq!(column.chunk_pos(), chunk);
                assert_eq!(column.local_x(), lx);
                assert_eq!(column.local_z(), lz);
            }
        }
    }

    #[test]
    fn column_chunk_boundary_differs() {
        assert_ne!(
            ColumnPos::new(0, 0).chunk_pos(),
            ColumnPos::new(16, 16).chunk_pos()
        );
    }

    // --- ChunkPos ---

    #[test]
    fn chunk_origin_matches_block_chunk() {
        for (x, z) in [(0, 0), (5, -7), (-3, 4), (-1, -1)] {
            let chunk = ChunkPos { x, z };
            let origin = chunk.origin();
            assert_eq!(origin.x, x * CHUNK_SIZE);
            assert_eq!(origin.z, z * CHUNK_SIZE);
            assert_eq!(origin.chunk_pos(), chunk);
        }
    }

    #[test]
    fn chunk_neighbors_4_are_four_distinct() {
        let chunk = ChunkPos { x: 10, z: -10 };
        let neighbors: Vec<_> = chunk.neighbors_4().collect();

        assert_eq!(neighbors.len(), 4);
        assert!(!neighbors.contains(&chunk));
        for (i, a) in neighbors.iter().enumerate() {
            for b in neighbors.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
            assert_eq!(
                (a.x - chunk.x).abs() + (a.z - chunk.z).abs(),
                1,
                "neighbor {a:?} is not adjacent to {chunk:?}"
            );
        }
    }

    #[test]
    fn chunk_chebyshev_is_symmetric_and_abs() {
        let a = ChunkPos { x: 0, z: 0 };
        let b = ChunkPos { x: -3, z: 5 };

        assert_eq!(a.chebyshev_to(b), 5);
        assert_eq!(b.chebyshev_to(a), 5);
        assert_eq!(a.chebyshev_to(a), 0);
    }
}
