//! 定义 Game 与 Client 共同遵守的体素区块几何契约。
//!
//! 这些值决定区块坐标换算、序列化数组长度和客户端网格尺寸；修改它们需要同步考虑
//! 旧存档兼容性，不能把它们当作普通渲染调参。

/// 单个区块在每条坐标轴上的体素数量。
pub const CHUNK_SIZE: usize = 16;

/// 单个区块包含的体素总数。
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
