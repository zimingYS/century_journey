use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::voxel::model::BlockModelConfig;
use crate::voxel::sound::BlockSoundConfig;
use crate::voxel::state::BlockStateDefinition;

/// 方块属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProperty{
    /// 方块编号
    pub identifier: String,
    /// 显示名称
    pub display_name: String,
    /// 渲染归类
    pub render_mode: RenderMode,
    /// 纹理
    pub textures: BlockTextureConfig,
    /// 硬度（破坏时间 = 硬度 × 基础时间）
    pub hardness: f32,
    /// 动态分配id
    #[serde(skip)]
    pub runtime_id: u16,

    /// 是否拥有物理碰撞
    #[serde(default)]
    pub is_solid: bool,

    /// 发光强度
    #[serde(default)]
    pub light_emission: u8,

    /// 方块模型配置
    #[serde(default)]
    pub model: BlockModelConfig,

    /// 方块音效配置
    #[serde(default)]
    pub sound: BlockSoundConfig,

    /// 方块状态定义
    #[serde(default)]
    pub states: BlockStateDefinition,

    /// 是否受重力影响
    #[serde(default)]
    pub has_gravity: bool,

    /// 掉落物（None = 自身，Some = 指定掉落物标识符）
    #[serde(default)]
    pub drop_identifier: Option<String>,

    /// 是否可被右键交互
    #[serde(default)]
    pub is_interactable: bool,

    /// 光照透射率 (0.0=不透光, 1.0=完全透光如空气)
    #[serde(default = "default_light_transmission")]
    pub light_transmission: f32,

    /// 方块行为类型标识（用于在注册时查找对应 Behavior）
    #[serde(default)]
    pub behavior_type: String,

    /// 方块标签 (自动填充到 TagRegistry)
    /// 示例: ["mineable/pickaxe", "stone_like"]
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for BlockProperty {
    fn default() -> Self {
        Self {
            identifier: String::new(),
            display_name: String::new(),
            render_mode: RenderMode::Opaque,
            is_solid: true,
            light_emission: 0,
            textures: BlockTextureConfig::default(),
            hardness: 1.0,
            runtime_id: 0,
            model: BlockModelConfig::default(),
            sound: BlockSoundConfig::default(),
            states: BlockStateDefinition::default(),
            has_gravity: false,
            drop_identifier: None,
            is_interactable: false,
            light_transmission: 0.0,
            behavior_type: String::new(),
            tags: Vec::new(),
        }
    }
}

fn default_light_transmission() -> f32 {
    if cfg!(debug_assertions) { 1.0 } else { 0.0 }
}

/// 方块渲染归类
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

impl Default for BlockTextureConfig {
    fn default() -> Self {
        Self {
            top: String::new(),
            bottom: None,
            north: None,
            south: None,
            west: None,
            east: None,
        }
    }
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