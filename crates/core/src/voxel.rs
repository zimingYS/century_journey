//! 体素与方块 id

/// 方块种类 ID（对应 content 中的 `BlockDef`）。
///
/// `BlockId` 是运行时使用的紧凑 ID，不是方块的持久化身份。
/// 方块的稳定身份由命名空间 ID，例如 `century_journey:stone`，定义。
///
/// 字段保持私有，避免外部代码绕过语义直接构造 ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BlockId(u16);

impl BlockId {
    /// 空气方块
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

/// 方块对应的模型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ShapeId(u8);

impl ShapeId {
    /// 默认方块为0
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

/// 单个体素方块状态
///
/// `VoxelState` 本身不解释具体含义，具体 bit 的语义由
/// 上层的方块定义决定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct VoxelState(u8);

impl VoxelState {
    /// 默认空状态
    pub const DEFAULT: Self = Self(0);

    /// 获取原始状态比特。
    #[inline]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// 通过原始比特创建状态。
    #[inline]
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }
}

/// 一个体素格的内容
///
/// 一个体素格占4字节
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Voxel {
    block: BlockId,
    shape: ShapeId,
    state: VoxelState,
}

impl Voxel {
    /// 空气体素
    pub const AIR: Self = Self {
        block: BlockId::AIR,
        shape: ShapeId::CUBE,
        state: VoxelState::DEFAULT,
    };

    /// 创建一个体素
    #[inline]
    pub const fn new(block: BlockId, shape: ShapeId, state: VoxelState) -> Self {
        Voxel {
            block,
            shape,
            state,
        }
    }

    /// 创建一个使用默认立方体形状和默认状态的体素。
    #[inline]
    pub const fn cube(block: BlockId) -> Self {
        Self::new(block, ShapeId::CUBE, VoxelState::DEFAULT)
    }

    /// 获取方块 ID。
    #[inline]
    pub const fn block(&self) -> BlockId {
        self.block
    }

    /// 获取形状 ID。
    #[inline]
    pub const fn shape(&self) -> ShapeId {
        self.shape
    }

    /// 获取状态。
    #[inline]
    pub const fn state(&self) -> VoxelState {
        self.state
    }

    /// 判断是否为空气。
    #[inline]
    pub const fn is_air(&self) -> bool {
        self.block.raw() == BlockId::AIR.raw()
    }
}
