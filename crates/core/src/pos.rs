//! 坐标类型

use crate::spec::{CHUNK_SIZE, SECTION_COUNT, SECTION_MIN_Y, SECTION_SIZE};

/// 世界坐标（方块整数坐标）
/// 表示世界中的方块坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// 区块切片内部坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionLocalPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

/// Chunk 在世界 XZ 平面上的坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Section 在世界中的坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionPos {
    pub chunk: ChunkPos,
    pub y: i32,
}

impl BlockPos {
    /// 创建一个新的世界方块坐标
    pub fn new(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    /// 将世界方块坐标转换为所属 Chunk 坐标
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(CHUNK_SIZE),
            z: self.z.div_euclid(CHUNK_SIZE),
        }
    }

    /// 将世界方块坐标转换为所属 Section 坐标
    pub fn section_pos(&self) -> SectionPos {
        SectionPos {
            chunk: self.chunk_pos(),
            y: self.y.div_euclid(SECTION_SIZE),
        }
    }

    /// 获取方块在所属 Section 内的局部坐标
    pub fn section_local(&self) -> SectionLocalPos {
        SectionLocalPos {
            x: self.x.rem_euclid(CHUNK_SIZE) as u8,
            y: self.y.rem_euclid(SECTION_SIZE) as u8,
            z: self.z.rem_euclid(CHUNK_SIZE) as u8,
        }
    }
}

impl SectionPos {
    /// 将局部 Section 坐标转换为整数索引
    pub fn index(&self) -> usize {
        let index = self.y - SECTION_MIN_Y;

        // 边界检查
        debug_assert!(
            (0..SECTION_COUNT).contains(&index),
            "section y out of range: {}",
            self.y
        );

        index as usize
    }

    /// 将整数索引转换为局部 Section 坐标
    pub fn from_index(chunk: ChunkPos, index: usize) -> Self {
        debug_assert!(
            index < SECTION_COUNT as usize,
            "section index out of range: {index}"
        );

        Self {
            chunk,
            y: index as i32 + SECTION_MIN_Y,
        }
    }

    /// 局部坐标获取世界方块坐标
    pub fn block_at(&self, local: SectionLocalPos) -> BlockPos {
        BlockPos {
            x: self.chunk.x * CHUNK_SIZE + local.x as i32,
            y: self.y * SECTION_SIZE + local.y as i32,
            z: self.chunk.z * CHUNK_SIZE + local.z as i32,
        }
    }

    /// 获取该 Section 底部对应的世界 Y 坐标
    pub fn block_y_start(&self) -> i32 {
        self.y * SECTION_SIZE
    }

    /// 将世界 Y 坐标转换为当前 Section 内的局部 Y
    pub fn to_section_y(&self, block_y: i32) -> u8 {
        block_y.rem_euclid(SECTION_SIZE) as u8
    }

    /// 获取一个世界方块在当前 Section 内的局部坐标
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
    /// 从世界方块坐标计算 Chunk 坐标
    pub fn from_block(block_pos: BlockPos) -> ChunkPos {
        block_pos.chunk_pos()
    }

    /// 获取 Chunk 在世界空间中的方块原点
    pub fn origin(&self) -> BlockPos {
        BlockPos {
            x: self.x * CHUNK_SIZE,
            y: 0,
            z: self.z * CHUNK_SIZE,
        }
    }

    /// 当前 Chunk 的水平四邻区块偏移
    pub const NEIGHBORS_4: [(i32, i32); 4] = [
        (0, -1), // north
        (1, 0),  // east
        (0, 1),  // south
        (-1, 0), // west
    ];

    /// 迭代当前 Chunk 的四个水平相邻 Chunk
    pub fn neighbors_4(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        Self::NEIGHBORS_4.iter().map(move |&(dx, dz)| ChunkPos {
            x: self.x + dx,
            z: self.z + dz,
        })
    }

    /// 计算两个 Chunk 在 XZ 平面上的切比雪夫距离
    /// 用于视距 / LOD 判断
    pub fn chebyshev_to(&self, other: ChunkPos) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dz = (self.z - other.z).unsigned_abs();
        dx.max(dz)
    }
}

impl SectionLocalPos {
    /// 将 Section 内局部坐标转换为线性数组下标
    pub fn index(&self) -> usize {
        let size = SECTION_SIZE as usize;
        let x = self.x as usize;
        let y = self.y as usize;
        let z = self.z as usize;
        x + z * size + y * size * size
    }
}
