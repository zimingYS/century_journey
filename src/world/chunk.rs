use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::constant::CHUNK_VOLUME;
use crate::voxel::types::VoxelType;

// 标记渲染的方块实体属于哪个区块
#[derive(Component)]
pub struct ChunkComponents {
    pub position: IVec3,
}

// 存储每个区块的方块
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct ChunkData {
    #[serde(with = "serde_arrays")]
    pub voxels: [u8; CHUNK_VOLUME],
}

impl ChunkData {
    #[inline]
    pub fn xyz_to_index(x: usize, y: usize, z: usize) -> usize{
        (y * 256) + (z * 16) + x
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> VoxelType {
        let idx = Self::xyz_to_index(x, y, z);
        let raw_id = self.voxels[idx];
        unsafe { std::mem::transmute(raw_id) }
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: VoxelType) {
        let idx = Self::xyz_to_index(x, y, z);
        self.voxels[idx] = voxel as u8;
    }

    // 安全读取局部方块
    pub fn get_voxel_safe(&self, x: i32, y: i32, z: i32) -> Option<u8>{
        if x <0 || x >= 16 || y < 0 || y >= 16 || z < 0 || z >= 16{
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