//! 定义仅影响客户端纹理和网格生成的渲染常量。

/// 单张方块贴图的像素边长。
pub const TILE_SIZE: u32 = 16;

/// 方块 atlas 每行和每列的瓦片数量。
pub const BLOCK_ATLAS_TILES_PER_ROW: u32 = 16;

/// 方块 atlas 中单个纹理层占用的瓦片数量。
pub const BLOCK_ATLAS_TILES_PER_LAYER: usize =
    (BLOCK_ATLAS_TILES_PER_ROW * BLOCK_ATLAS_TILES_PER_ROW) as usize;
