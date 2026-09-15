//! 实现简单的方块定义
//! 待世界生成稳定后修改方块定义，包括方块的定义和纹理映射等

use crate::voxel::BlockId;

/// 方块定义的实现(临时)
impl BlockId {
    // 一些方块的定义
    pub const STONE: BlockId = BlockId::from_raw(1);
    pub const DIRT: BlockId = BlockId::from_raw(2);
    pub const GRASS: BlockId = BlockId::from_raw(3);
    pub const SAND: BlockId = BlockId::from_raw(4);
    pub const WATER: BlockId = BlockId::from_raw(5);

    /// 判断是否为空气方块
    #[inline]
    pub const fn is_air(&self) -> bool {
        self.raw() == 0
    }

    /// 判断方块是否遮挡相邻方块的面
    #[inline]
    pub const fn occludes(&self) -> bool {
        !self.is_air()
    }
}
