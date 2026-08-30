//! 加载生存物品栏专属纹理，并提供九宫格皮肤构造工具。

use bevy::prelude::*;
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};

use crate::engine::asset::identifier::asset_id;
use crate::engine::asset::manager::AssetManager;

/// 物品槽位九宫格边框厚度；素材 96x97，像素风边框。
pub const SURVIVAL_SLOT_SLICE: f32 = 8.0;

/// 生存物品栏专属纹理句柄集合。
///
/// 全部通过 [`AssetManager`] 加载以复用最近邻采样，保证像素风不糊。
#[derive(Resource, Debug, Clone)]
pub struct SurvivalUiAssets {
    /// 主面板整图背景（设计稿等比拉伸，不切九宫格）。
    pub panel: Handle<Image>,
    /// 物品槽位九宫格。
    pub slot: Handle<Image>,
    /// 快捷栏选中框（整图，不切九宫格）。
    pub hotbar_selection: Handle<Image>,
}

/// 构造统一拉伸策略的九宫格图片节点。
pub fn survival_sliced_image_node(image: Handle<Image>, slice: f32) -> ImageNode {
    ImageNode {
        image,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(slice),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

/// 加载生存物品栏纹理资源。
///
/// 在 Startup 链中先于界面生成系统执行，让布局代码直接拿到句柄。
pub fn load_survival_ui_assets_system(
    mut commands: Commands,
    mut asset_manager: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
) {
    let load = |manager: &mut AssetManager, path: &str| {
        manager.texture(&asset_id(path), &asset_server).handle
    };
    commands.insert_resource(SurvivalUiAssets {
        panel: load(&mut asset_manager, "textures/ui/survival/panel"),
        slot: load(&mut asset_manager, "textures/ui/survival/slot"),
        hotbar_selection: load(&mut asset_manager, "textures/ui/survival/hotbar_selection"),
    });
}
