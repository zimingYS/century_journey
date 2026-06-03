use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;

/// 区块内单个坐标(每列)共享的上下文
#[derive(Debug,Clone)]
pub struct ColumnContext{
    /// 世界 x 坐标
    pub world_x: i32,
    /// 世界 z 坐标
    pub world_z: i32,
    /// 温度 (0.0=极寒, 1.0=极热)
    pub temperature: f64,
    /// 湿度 (0.0=极干, 1.0=极湿)
    pub humidity: f64,
    /// 生物群系类型索引
    pub biome_index: u8,
    /// 地形基础高度
    pub base_height: i32,
    /// 地形粗糙度（影响山丘起伏）
    pub roughness: f64,
}

/// 整个区块的生成上下文
#[derive(Debug, Clone)]
pub struct ChunkGenContext {
    /// 区块坐标
    pub chunk_pos: IVec3,
    /// 每列的上下文（x,z → ColumnContext）
    pub columns: Vec<ColumnContext>,
}

impl ChunkGenContext {
    pub fn new(chunk_pos: IVec3) -> Self {
        Self {
            chunk_pos,
            columns: Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE),
        }
    }

    /// 按局部坐标获取列上下文
    pub fn get_column(&self, local_x: usize, local_z: usize) -> &ColumnContext {
        &self.columns[local_x * CHUNK_SIZE + local_z]
    }
}