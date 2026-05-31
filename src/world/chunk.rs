use crate::core::constant::world::CHUNK_VOLUME;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// 标记渲染的方块实体属于哪个区块
#[derive(Component)]
pub struct ChunkComponents {
    pub position: IVec3,
}

// 存储每个区块的方块
#[derive(Serialize, Deserialize, Clone)]
pub struct ChunkData {
    #[serde(with = "serde_arrays")]
    pub voxels: [u16; CHUNK_VOLUME],
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
    pub fn xyz_to_index(x: usize, y: usize, z: usize) -> usize{
        (y * 256) + (z * 16) + x
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> u16 {
        let idx = Self::xyz_to_index(x, y, z);
        self.voxels[idx]
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel_id: u16) {
        let idx = Self::xyz_to_index(x, y, z);
        self.voxels[idx] = voxel_id;
    }

    // 安全读取局部方块
    pub fn get_voxel_safe(&self, x: i32, y: i32, z: i32) -> Option<u16> {
        if x < 0 || x >= 16 || y < 0 || y >= 16 || z < 0 || z >= 16 {
            return None;
        }
        let idx = Self::xyz_to_index(x as usize, y as usize, z as usize);
        Some(self.voxels[idx])
    }
}

// 标记区块状态
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    /// 等待或正在进行计算地形噪点数据
    GeneratingData,
    /// 方块数据计算完毕，等待生成3D网络
    DataReady,
    /// 正在计算3D顶点
    GeneratingMesh,
    /// 正在渲染
    Rendered,
}