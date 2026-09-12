//! 体素与方块 id

/// 方块种类 id（对应 content 中的 BlockDef）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u16);

impl BlockId {
    pub const AIR: BlockId = BlockId(0);
}

/// 一个体素格的内容
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel {
    pub id: BlockId,
}

