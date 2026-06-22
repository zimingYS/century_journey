use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 方块几何模型类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockModel {
    /// 标准完整立方体
    Cube,
    /// 十字形
    Cross,
    /// 薄板
    Slab {
        /// 厚度（0.0~1.0，相对于标准方块高度）
        thickness: f32,
    },
    /// 自定义多面模型
    Custom {
        /// 每个面的定义
        faces: Vec<CustomFace>,
    },
}

impl Default for BlockModel {
    fn default() -> Self {
        Self::Cube
    }
}

/// 自定义模型中的单个面
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFace {
    /// 面的四个顶点
    pub vertices: [[f32; 3]; 4],
    /// 面的法线方向
    pub normal: [f32; 3],
    /// 纹理槽位
    pub texture_slot: String,
    /// 面的环境光遮蔽亮度（0.0~1.0，1.0=无遮蔽）
    pub ambient_occlusion: f32,
}

/// 方块模型与纹理的绑定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockModelConfig {
    /// 方块使用的模型
    pub model: BlockModel,
    /// 模型是否绕 Y 轴随机旋转（用于花草打破单调感）
    pub random_rotation: bool,
    /// 模型是否在方块中心偏移（用于小方块如睡莲）
    pub centered: bool,
}

impl Default for BlockModelConfig {
    fn default() -> Self {
        Self {
            model: BlockModel::Cube,
            random_rotation: false,
            centered: false,
        }
    }
}

/// 生成十字形模型的6个面顶点
pub fn generate_cross_vertices(x: f32, y: f32, z: f32) -> Vec<[[f32; 3]; 4]> {
    let offset = 0.15;
    vec![
        [
            [x + offset, y, z + offset],
            [x + 1.0 - offset, y, z + 1.0 - offset],
            [x + 1.0 - offset, y + 1.0, z + 1.0 - offset],
            [x + offset, y + 1.0, z + offset],
        ],
        [
            [x + 1.0 - offset, y, z + 1.0 - offset],
            [x + offset, y, z + offset],
            [x + offset, y + 1.0, z + offset],
            [x + 1.0 - offset, y + 1.0, z + 1.0 - offset],
        ],
        [
            [x + 1.0 - offset, y, z + offset],
            [x + offset, y, z + 1.0 - offset],
            [x + offset, y + 1.0, z + 1.0 - offset],
            [x + 1.0 - offset, y + 1.0, z + offset],
        ],
        [
            [x + offset, y, z + 1.0 - offset],
            [x + 1.0 - offset, y, z + offset],
            [x + 1.0 - offset, y + 1.0, z + offset],
            [x + offset, y + 1.0, z + 1.0 - offset],
        ],
    ]
}
