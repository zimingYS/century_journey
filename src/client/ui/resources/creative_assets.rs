//! 加载创造物品栏专属纹理，并提供九宫格皮肤构造工具。

use bevy::prelude::*;
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};

use crate::engine::asset::identifier::asset_id;
use crate::engine::asset::manager::AssetManager;

/// 物品框九宫格边框厚度；素材 91x96，四角带铆钉装饰。
pub const SLOT_SLICE: f32 = 10.0;
/// 分类标签九宫格边框厚度；素材约 268x76。
pub const TAB_SLICE: f32 = 12.0;
/// 搜索框九宫格边框厚度；素材 461x68。
pub const SEARCH_BOX_SLICE: f32 = 12.0;

/// 创造物品栏专属纹理句柄集合。
///
/// 全部通过 [`AssetManager`] 加载以复用最近邻采样，保证像素风不糊。
#[derive(Resource, Debug, Clone)]
pub struct CreativeUiAssets {
    /// 主面板整图背景（设计稿等比拉伸，不切九宫格）。
    pub panel: Handle<Image>,
    /// 物品槽位九宫格。
    pub slot: Handle<Image>,
    /// 搜索框九宫格。
    pub search_box: Handle<Image>,
    /// 搜索放大镜图标。
    pub search_icon: Handle<Image>,
    /// 关闭按钮图标。
    pub close: Handle<Image>,
    /// 选中状态分类标签九宫格。
    pub tab_active: Handle<Image>,
    /// 未选中状态分类标签九宫格。
    pub tab_inactive: Handle<Image>,
    /// 左翻页按钮（分类列表底部，整图拉伸非九宫格）。
    pub pager_left: Handle<Image>,
    /// 右翻页按钮（分类列表底部，整图拉伸非九宫格）。
    pub pager_right: Handle<Image>,
}

/// 构造统一拉伸策略的九宫格图片节点。
pub fn sliced_image_node(image: Handle<Image>, slice: f32) -> ImageNode {
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

/// 加载创造物品栏纹理资源。
///
/// 在 Startup 链中先于界面生成系统执行，让布局代码直接拿到句柄。
pub fn load_creative_ui_assets_system(
    mut commands: Commands,
    mut asset_manager: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
) {
    let load = |manager: &mut AssetManager, path: &str| {
        manager.texture(&asset_id(path), &asset_server).handle
    };
    commands.insert_resource(CreativeUiAssets {
        panel: load(&mut asset_manager, "textures/ui/creative/panel"),
        slot: load(&mut asset_manager, "textures/ui/creative/slot"),
        search_box: load(&mut asset_manager, "textures/ui/creative/search_box"),
        search_icon: load(&mut asset_manager, "textures/ui/creative/search_icon"),
        close: load(&mut asset_manager, "textures/ui/creative/close"),
        tab_active: load(&mut asset_manager, "textures/ui/creative/tab_active"),
        tab_inactive: load(&mut asset_manager, "textures/ui/creative/tab_inactive"),
        pager_left: load(&mut asset_manager, "textures/ui/creative/pager_left"),
        pager_right: load(&mut asset_manager, "textures/ui/creative/pager_right"),
    });
}
