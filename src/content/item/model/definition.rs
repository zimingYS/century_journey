use crate::content::item::definition::{ItemCategory, ItemDefinition};
use crate::content::item::texture::icon::IconDefinition;
use crate::shared::held_item::HeldRenderDefinition;
use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};

use super::display::ItemModelDisplay;

/// 可序列化的物品模型定义。
///
/// 该类型属于 content 层，只描述模型来源、贴图和 display 配置，不直接持有 Bevy Mesh / Material。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemModelDefinition {
    /// 模型标识符；从文件加载时可省略，由 loader 根据路径补齐。
    #[serde(default)]
    pub identifier: Option<Identifier>,
    /// 模型类型和对应参数。
    #[serde(default, flatten)]
    pub kind: ItemModelKind,
    /// 不同展示场景下的变换。
    #[serde(default)]
    pub display: ItemModelDisplay,
}

/// 物品模型来源类型。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemModelKind {
    /// 空模型，适合 air 或暂不显示的物品。
    #[default]
    Empty,
    /// 方块 cube 模型，运行时从 BlockRegistry 和 atlas 材质构建。
    Block {
        /// 对应方块标识符。
        block: Identifier,
    },
    /// 从 2D 贴图生成挤出模型。
    Generated {
        /// 物品贴图标识符。
        texture: Identifier,
        /// 挤出厚度。
        #[serde(default = "default_generated_thickness")]
        thickness: f32,
    },
    /// 自定义模型占位，后续可接 glTF 或 JSON model。
    Custom {
        /// 自定义模型资源路径。
        path: String,
    },
}

/// 生成式物品模型的默认挤出厚度。
fn default_generated_thickness() -> f32 {
    0.05
}

impl ItemModelDefinition {
    /// 创建空模型定义。
    pub fn empty() -> Self {
        Self {
            identifier: None,
            kind: ItemModelKind::Empty,
            display: ItemModelDisplay::default(),
        }
    }

    /// 创建方块模型定义。
    pub fn block(block: Identifier) -> Self {
        Self {
            identifier: None,
            kind: ItemModelKind::Block { block },
            display: ItemModelDisplay::block_defaults(),
        }
    }

    /// 创建 generated item 挤出模型定义。
    pub fn generated(texture: Identifier, thickness: f32, handheld: bool) -> Self {
        Self {
            identifier: None,
            kind: ItemModelKind::Generated { texture, thickness },
            display: ItemModelDisplay::generated_defaults(handheld),
        }
    }

    /// 创建自定义模型定义。
    pub fn custom(path: impl Into<String>) -> Self {
        Self {
            identifier: None,
            kind: ItemModelKind::Custom { path: path.into() },
            display: ItemModelDisplay::generated_defaults(false),
        }
    }

    /// 设置模型标识符。
    pub fn with_identifier(mut self, identifier: Identifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    /// 覆盖 display 配置。
    pub fn with_display(mut self, display: ItemModelDisplay) -> Self {
        self.display = display;
        self
    }

    /// 根据旧 ItemDefinition 自动推导 fallback 模型定义。
    ///
    /// 这是过渡期兼容层：外部没有显式模型 JSON 时，仍能从 category / icon / held_renderer 得到可渲染结果。
    pub fn fallback_for_item_definition(definition: &ItemDefinition) -> Self {
        let handheld = matches!(
            definition.category,
            ItemCategory::Tool | ItemCategory::Weapon
        );
        let display = if definition.category == ItemCategory::Block {
            ItemModelDisplay::block_defaults()
        } else {
            ItemModelDisplay::generated_defaults(handheld)
        };

        let model = match &definition.held_renderer {
            HeldRenderDefinition::Model { path } => Self::custom(path.clone()),
            HeldRenderDefinition::Block => Self::block(block_identifier_for(definition)),
            HeldRenderDefinition::FlatItem { thickness } => {
                Self::generated(texture_identifier_for(definition), *thickness, handheld)
            }
            HeldRenderDefinition::Empty => {
                if definition.category == ItemCategory::Block
                    || definition.placeable_block.is_some()
                    || definition.icon.as_block_id().is_some()
                {
                    Self::block(block_identifier_for(definition))
                } else {
                    Self::generated(
                        texture_identifier_for(definition),
                        default_generated_thickness(),
                        handheld,
                    )
                }
            }
        };

        model
            .with_identifier(definition.identifier.clone())
            .with_display(display)
    }
}

/// 推导方块模型使用的方块标识符。
fn block_identifier_for(definition: &ItemDefinition) -> Identifier {
    definition
        .placeable_block
        .clone()
        .or_else(|| definition.icon.as_block_id().cloned())
        .unwrap_or_else(|| definition.identifier.clone())
}

/// 推导 generated item 使用的贴图标识符。
fn texture_identifier_for(definition: &ItemDefinition) -> Identifier {
    match &definition.icon {
        IconDefinition::Texture(path) => {
            Identifier::parse(path).unwrap_or_else(|_| definition.identifier.clone())
        }
        IconDefinition::Block(id) => id.clone(),
    }
}
