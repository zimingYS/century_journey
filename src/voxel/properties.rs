use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 方块属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProperty{
    /// 方块编号
    pub identifier: String,
    /// 显示名称
    pub display_name: String,
    /// 渲染归类
    pub render_mode: RenderMode,
    /// 是否拥有物理碰撞
    pub is_solid: bool,
    /// 发光强度
    pub light_emission: u8,
    /// 纹理
    pub textures: BlockTextureConfig,
    /// 拓展属性
    pub hardness: f32,
    /// 动态分配id
    #[serde(skip)]
    pub runtime_id: u16,
}

/// 方块渲染归类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RenderMode{
    /// 普通不透明方块
    Opaque,
    /// 半透明方块
    Transparent,
    /// 透明剔除方块
    Cutout,
    /// 自定义模型方块
    CustomMesh,
}

/// 方块纹理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTextureConfig {
    // 顶面为首要索引
    pub top: String,
    pub bottom: Option<String>,
    pub north: Option<String>,
    pub south: Option<String>,
    pub west: Option<String>,
    pub east: Option<String>,
}

impl BlockTextureConfig {
    /// 计算贴图路径
    pub fn get_face_texture(&self, face_idx: usize) -> &str {
        match face_idx {
            0 => &self.top,
            1 => self.bottom.as_deref().unwrap_or(&self.top),
            2 => self.west.as_deref().or(self.north.as_deref()).unwrap_or(&self.top),
            3 => self.east.as_deref().or(self.north.as_deref()).unwrap_or(&self.top),
            4 => self.south.as_deref().or(self.north.as_deref()).unwrap_or(&self.top),
            5 => self.north.as_deref().unwrap_or(&self.top),
            _ => unreachable!("未知方块类型！"),
        }
    }
}