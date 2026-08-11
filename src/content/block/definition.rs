//! 定义方块属性、渲染模式和纹理配置。

use crate::content::block::model::BlockModelConfig;
use crate::content::block::placement::BlockPlacementConfig;
use crate::content::block::sound::BlockSoundConfig;
use crate::content::block::state::BlockStateDefinition;
use crate::content::item::definition::tool::ToolType;
use crate::shared::identifier::Identifier;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 方块发光定义；`emission = 0` 表示不发光。
///
/// 体素光传播消费 `emission`、`color` 与 `range`，客户端可以额外把光源
/// 映射为有预算限制的 Bevy `PointLight`；`casts_shadow` 只控制后者。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BlockLightDef {
    /// 发光强度（0-15 光级）。
    pub emission: u8,
    /// 发光颜色（线性 RGB）。
    pub color: [f32; 3],
    /// 最大传播半径（方块格）。
    pub range: u8,
    /// 客户端映射为 Bevy 实体光时是否启用阴影贴图。
    pub casts_shadow: bool,
}

impl Default for BlockLightDef {
    fn default() -> Self {
        Self {
            emission: 0,
            color: [1.0, 0.80, 0.60],
            range: 14,
            casts_shadow: false,
        }
    }
}

impl BlockLightDef {
    /// 旧平铺 `light_emission` 字段迁移用的默认颜色（暖白）。
    const LEGACY_FALLBACK_COLOR: [f32; 3] = [1.0, 0.69, 0.31];

    /// 由旧 `light_emission` 单值构造等价的发光定义。
    pub fn from_legacy_emission(emission: u8) -> Self {
        Self {
            emission,
            color: Self::LEGACY_FALLBACK_COLOR,
            range: 14,
            casts_shadow: false,
        }
    }
}

/// 方块属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProperty {
    /// 方块编号
    pub identifier: Identifier,
    /// 显示名称
    pub display_name: String,
    /// 渲染归类
    pub render_mode: RenderMode,
    /// 纹理
    pub textures: BlockTextureConfig,
    /// 硬度（破坏时间 = 硬度 × 基础时间）
    pub hardness: f32,

    #[serde(default)]
    pub required_tool: Option<ToolType>,

    /// 可获得挖掘效率加成、但不作为掉落必要条件的工具类型。
    #[serde(default)]
    pub effective_tool: Option<ToolType>,

    #[serde(default)]
    pub harvest_level: u8,

    /// 是否拥有物理碰撞
    #[serde(default)]
    pub is_solid: bool,

    /// 发光强度（旧平铺字段；新定义优先使用嵌套 `light` 对象）。
    #[serde(default)]
    pub light_emission: u8,

    /// 方块发光定义（嵌套对象，替代平铺 light_emission）。
    #[serde(default)]
    pub light: Option<BlockLightDef>,

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
    pub drop_identifier: Option<Identifier>,

    /// 是否可被右键交互
    #[serde(default)]
    pub is_interactable: bool,

    /// 白光透射率（0.0 = 不透光，1.0 = 完全透光）。
    ///
    /// 非实心不等于透光；空气、植物、水等内容仍必须显式声明该字段，
    /// 避免自定义碰撞形状意外改变光传播语义。
    #[serde(default = "default_light_transmission")]
    pub light_transmission: f32,

    /// RGB 透射系数；存在时覆盖 `light_transmission` 的白光系数。
    ///
    /// 例如红色染色玻璃可声明 `[0.9, 0.08, 0.05]`，让天空光和方块光
    /// 都沿传播路径逐通道滤色。
    #[serde(default)]
    pub light_filter: Option<[f32; 3]>,

    /// 方块行为类型标识（用于在注册时查找对应 Behavior）
    #[serde(default)]
    pub behavior_type: String,

    /// 方块标签 (自动填充到 TagRegistry)
    /// 示例: ["mineable/pickaxe", "stone_like"]
    #[serde(default)]
    pub tags: Vec<String>,

    /// 方块放置规则。
    #[serde(default)]
    pub placement: BlockPlacementConfig,
}

impl Default for BlockProperty {
    fn default() -> Self {
        Self {
            identifier: Identifier::new("century_journey", ""),
            display_name: String::new(),
            render_mode: RenderMode::Opaque,
            is_solid: true,
            light_emission: 0,
            light: None,
            textures: BlockTextureConfig::default(),
            hardness: 1.0,
            required_tool: None,
            effective_tool: None,
            harvest_level: 0,
            model: BlockModelConfig::default(),
            sound: BlockSoundConfig::default(),
            states: BlockStateDefinition::default(),
            has_gravity: false,
            drop_identifier: None,
            is_interactable: false,
            light_transmission: 0.0,
            light_filter: None,
            behavior_type: String::new(),
            tags: Vec::new(),
            placement: BlockPlacementConfig::default(),
        }
    }
}

fn default_light_transmission() -> f32 {
    0.0
}

#[cfg(test)]
#[path = "../../../tests/unit/content/block/definition.rs"]
mod tests;

/// 方块渲染归类
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RenderMode {
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            2 => self
                .west
                .as_deref()
                .or(self.north.as_deref())
                .unwrap_or(&self.top),
            3 => self
                .east
                .as_deref()
                .or(self.north.as_deref())
                .unwrap_or(&self.top),
            4 => self
                .south
                .as_deref()
                .or(self.north.as_deref())
                .unwrap_or(&self.top),
            5 => self.north.as_deref().unwrap_or(&self.top),
            _ => unreachable!("未知方块类型！"),
        }
    }
}
