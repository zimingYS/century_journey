use bevy::prelude::*;

use crate::content::item::model::ItemModelDisplay;

use super::display::ItemDisplayContext;

/// 已烘焙的物品模型。
///
/// 这是运行时渲染层可以直接使用的模型：它已经持有 Bevy Mesh / Material 句柄，但仍保留 display 变换以适配不同展示场景。
#[derive(Debug, Clone)]
pub struct BakedItemModel {
    /// 模型包含的所有可渲染部件。
    pub parts: Vec<BakedItemModelPart>,
    /// 各展示场景对应的模型变换。
    pub display: ItemModelDisplay,
}

/// 已烘焙物品模型中的一个 mesh 部件。
#[derive(Debug, Clone)]
pub struct BakedItemModelPart {
    /// 部件名称，主要用于调试和实体命名。
    pub name: String,
    /// Bevy mesh 句柄。
    pub mesh: Handle<Mesh>,
    /// Bevy StandardMaterial 句柄。
    pub material: Handle<StandardMaterial>,
    /// 部件相对于模型根节点的局部变换。
    pub transform: Transform,
}

impl BakedItemModel {
    /// 创建空模型。
    pub fn empty(display: ItemModelDisplay) -> Self {
        Self {
            parts: Vec::new(),
            display,
        }
    }

    /// 创建只有一个 mesh 部件的模型。
    pub fn single(
        name: impl Into<String>,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        display: ItemModelDisplay,
    ) -> Self {
        Self {
            parts: vec![BakedItemModelPart {
                name: name.into(),
                mesh,
                material,
                transform,
            }],
            display,
        }
    }

    /// 根据渲染场景取得模型根节点变换。
    pub fn display_transform(&self, context: ItemDisplayContext) -> Transform {
        self.display.transform_for(context.model_target())
    }

    /// 判断模型是否没有任何可渲染部件。
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}
