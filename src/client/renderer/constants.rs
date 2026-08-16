//! 定义仅影响客户端纹理和网格生成的渲染常量。

/// 方块 atlas 单张贴图的像素边长。
///
/// 项目内方块纹理全部 32×32，这里保持 1:1 原样写入图集，避免像素降采样后
/// 看起来像被"压缩"。
pub const BLOCK_TILE_SIZE: u32 = 32;

/// 方块 atlas 每行和每列的瓦片数量。
pub const BLOCK_ATLAS_TILES_PER_ROW: u32 = 16;

/// 方块 atlas 中单个纹理层占用的瓦片数量。
pub const BLOCK_ATLAS_TILES_PER_LAYER: usize =
    (BLOCK_ATLAS_TILES_PER_ROW * BLOCK_ATLAS_TILES_PER_ROW) as usize;

/// 水面相对完整方块顶面的下沉距离；近景水面和远景海面必须保持一致。
pub const WATER_SURFACE_INSET: f32 = 0.12;
