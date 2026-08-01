//! 定义固定尺寸区块的体素存储及局部坐标访问方法。

use crate::shared::voxel::{CHUNK_SIZE, CHUNK_VOLUME};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 标记渲染的方块实体属于哪个区块
#[derive(Component)]
pub struct ChunkComponents {
    /// 区块在世界区块网格中的整数坐标，而非方块或渲染坐标。
    pub position: IVec3,
}

/// 存储每个区块的方块
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkData {
    /// 按固定局部坐标顺序存储的运行时方块编号；该顺序也是存档编码契约。
    #[serde(with = "serde_arrays")]
    pub voxels: [u16; CHUNK_VOLUME],
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkData {
    /// 创建空白区块
    pub fn new() -> Self {
        Self {
            voxels: [0u16; CHUNK_VOLUME],
        }
    }

    /// 扁平化 3D 坐标到一维数组索引
    #[inline]
    pub fn xyz_to_index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x
    }

    /// 读取区块局部坐标处的运行时方块编号。
    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> u16 {
        let idx = Self::xyz_to_index(x, y, z);
        self.voxels[idx]
    }

    /// 写入区块局部坐标；调用方必须保证坐标位于区块范围内。
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel_id: u16) {
        let idx = Self::xyz_to_index(x, y, z);
        self.voxels[idx] = voxel_id;
    }

    /// 安全读取局部方块
    pub fn get_voxel_safe(&self, x: i32, y: i32, z: i32) -> Option<u16> {
        if x < 0
            || x >= CHUNK_SIZE as i32
            || y < 0
            || y >= CHUNK_SIZE as i32
            || z < 0
            || z >= CHUNK_SIZE as i32
        {
            return None;
        }
        let idx = Self::xyz_to_index(x as usize, y as usize, z as usize);
        Some(self.voxels[idx])
    }
}

/// 区块生成、结构和渲染任务的唯一生命周期阶段。
/// `GenerationPipeline` 只执行纯生成步骤，不再维护第二套阶段状态。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    /// 空区块
    Empty,
    /// 初始状态，正在生成地形基础地形数据
    GeneratingTerrain,
    /// 区块存档存在但无法安全读取；保持空白并等待显式修复，禁止重新生成覆盖。
    LoadFailed,
    /// 基础地形生成完成，等待进入结构生成阶段
    TerrainReady,
    /// 生成结构
    GeneratingStructure,
    /// 结构生成完毕
    StructureReady,
    /// 正在计算3D顶点
    GeneratingMesh,
    /// 正在渲染
    Rendered,
}
