//! 体素与方块 id

/// 方块种类 ID（对应 content 中的 `BlockDef`）
///
/// 运行时紧凑 ID，非持久化身份
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BlockId(u16);

impl BlockId {
    /// 空气
    pub const AIR: BlockId = BlockId(0);

    #[inline]
    pub const fn raw(&self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn from_raw(raw: u16) -> Self {
        BlockId(raw)
    }
}

/// 方块模型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ShapeId(u8);

impl ShapeId {
    /// 默认立方体
    pub const CUBE: Self = Self(0);

    #[inline]
    pub const fn raw(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }
}

/// 方块状态位（语义由上层方块定义解释）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct VoxelState(u8);

impl VoxelState {
    /// 默认状态
    pub const DEFAULT: Self = Self(0);

    /// 原始状态位
    #[inline]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// 由原始状态位构造
    #[inline]
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }
}

/// 一个体素格的内容（4 字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Voxel {
    block: BlockId,
    shape: ShapeId,
    state: VoxelState,
}

impl Voxel {
    /// 空气
    pub const AIR: Self = Self {
        block: BlockId::AIR,
        shape: ShapeId::CUBE,
        state: VoxelState::DEFAULT,
    };

    /// 构造体素
    #[inline]
    pub const fn new(block: BlockId, shape: ShapeId, state: VoxelState) -> Self {
        Voxel {
            block,
            shape,
            state,
        }
    }

    /// 构造默认形状与状态的体素
    #[inline]
    pub const fn cube(block: BlockId) -> Self {
        Self::new(block, ShapeId::CUBE, VoxelState::DEFAULT)
    }

    /// 方块 ID
    #[inline]
    pub const fn block(&self) -> BlockId {
        self.block
    }

    /// 形状 ID
    #[inline]
    pub const fn shape(&self) -> ShapeId {
        self.shape
    }

    /// 状态
    #[inline]
    pub const fn state(&self) -> VoxelState {
        self.state
    }

    /// 是否空气
    #[inline]
    pub const fn is_air(&self) -> bool {
        self.block.raw() == BlockId::AIR.raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_id_raw_round_trip() {
        for raw in [0u16, 1, 2, 255, 256, u16::MAX] {
            assert_eq!(BlockId::from_raw(raw).raw(), raw);
        }
        assert_eq!(BlockId::AIR.raw(), 0);
    }

    #[test]
    fn shape_id_raw_round_trip() {
        for raw in [0u8, 1, 127, 255] {
            assert_eq!(ShapeId::from_raw(raw).raw(), raw);
        }
        assert_eq!(ShapeId::CUBE.raw(), 0);
    }

    #[test]
    fn voxel_state_raw_round_trip() {
        for raw in [0u8, 1, 0b1010_0101, 255] {
            assert_eq!(VoxelState::from_raw(raw).raw(), raw);
        }
        assert_eq!(VoxelState::DEFAULT.raw(), 0);
    }

    #[test]
    fn air_voxel_is_air() {
        assert!(Voxel::AIR.is_air());
        assert_eq!(Voxel::AIR.block(), BlockId::AIR);
        assert_eq!(Voxel::AIR.shape(), ShapeId::CUBE);
        assert_eq!(Voxel::AIR.state(), VoxelState::DEFAULT);
    }

    #[test]
    fn cube_uses_default_shape_and_state() {
        let stone = BlockId::from_raw(7);
        let voxel = Voxel::cube(stone);

        assert_eq!(voxel.block(), stone);
        assert_eq!(voxel.shape(), ShapeId::CUBE);
        assert_eq!(voxel.state(), VoxelState::DEFAULT);
    }

    #[test]
    fn non_air_block_is_not_air() {
        assert!(!Voxel::cube(BlockId::from_raw(1)).is_air());
        assert!(!Voxel::cube(BlockId::from_raw(4096)).is_air());
    }

    #[test]
    fn new_preserves_all_three_fields() {
        let block = BlockId::from_raw(42);
        let shape = ShapeId::from_raw(3);
        let state = VoxelState::from_raw(0b1100);

        let voxel = Voxel::new(block, shape, state);
        assert_eq!(voxel.block(), block);
        assert_eq!(voxel.shape(), shape);
        assert_eq!(voxel.state(), state);
    }

    #[test]
    fn voxel_is_four_bytes() {
        assert_eq!(std::mem::size_of::<Voxel>(), 4);
        assert_eq!(std::mem::size_of::<BlockId>(), 2);
        assert_eq!(std::mem::size_of::<ShapeId>(), 1);
        assert_eq!(std::mem::size_of::<VoxelState>(), 1);
    }

    #[test]
    fn voxel_equality_and_hash_follow_fields() {
        let a = Voxel::new(BlockId::from_raw(5), ShapeId::CUBE, VoxelState::DEFAULT);
        let b = Voxel::new(BlockId::from_raw(5), ShapeId::CUBE, VoxelState::DEFAULT);
        let c = Voxel::new(BlockId::from_raw(5), ShapeId::CUBE, VoxelState::from_raw(1));

        assert_eq!(a, b);
        assert_ne!(a, c, "state difference must break equality");

        let mut set = std::collections::HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
    }
}
