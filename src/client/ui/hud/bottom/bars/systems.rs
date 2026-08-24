//! 底部状态条的图标加载与容器生成系统。

use bevy::prelude::*;

use crate::client::ui::hud::bottom::BottomHud;
use crate::client::ui::hud::bottom::bars::components::{BarsHud, LeftBarsHud, RightBarsHud};
use crate::client::ui::hud::bottom::bars::layout::hud_hotbar_outer_width;
use crate::client::ui::hud::bottom::bars::resources::HudStatusIconAssets;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::engine::asset::identifier::asset_id;
use crate::engine::asset::manager::AssetManager;

/// 加载 HUD 状态图标资源。
///
/// 通过 AssetManager::texture 加载可以复用项目的像素纹理加载规则，保持最近邻采样。
pub fn load_hud_status_icon_assets_system(
    mut icons: ResMut<HudStatusIconAssets>,
    mut asset_manager: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
) {
    icons.heart_full = asset_manager
        .texture(&asset_id("textures/ui/hud/heart_full"), &asset_server)
        .handle;
    icons.heart_half = asset_manager
        .texture(&asset_id("textures/ui/hud/heart_half"), &asset_server)
        .handle;
    icons.heart_empty = asset_manager
        .texture(&asset_id("textures/ui/hud/heart_empty"), &asset_server)
        .handle;
    icons.hunger_full = asset_manager
        .texture(&asset_id("textures/ui/hud/hunger_full"), &asset_server)
        .handle;
    icons.hunger_half = asset_manager
        .texture(&asset_id("textures/ui/hud/hunger_half"), &asset_server)
        .handle;
    icons.hunger_empty = asset_manager
        .texture(&asset_id("textures/ui/hud/hunger_empty"), &asset_server)
        .handle;
    icons.thirst_full = asset_manager
        .texture(&asset_id("textures/ui/hud/thirst_full"), &asset_server)
        .handle;
    icons.thirst_half = asset_manager
        .texture(&asset_id("textures/ui/hud/thirst_half"), &asset_server)
        .handle;
    icons.thirst_empty = asset_manager
        .texture(&asset_id("textures/ui/hud/thirst_empty"), &asset_server)
        .handle;
}

/// 生成底部状态条容器。
pub fn spawn_bars_hud_system(
    mut commands: Commands,
    bottom_hud: Query<Entity, With<BottomHud>>,
    theme: Res<UiTheme>,
) {
    let Ok(bottom_entity) = bottom_hud.single() else {
        log::error!("[HUD] 状态条挂载失败：底部区域节点未生成");
        return;
    };

    commands.entity(bottom_entity).with_children(|parent| {
        parent
            .spawn((
                BarsHud,
                Name::new("BarsHud"),
                Node {
                    width: Val::Px(hud_hotbar_outer_width(&theme)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|bars| {
                bars.spawn((
                    LeftBarsHud,
                    Name::new("LeftBarsHud"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ));
                bars.spawn((
                    RightBarsHud,
                    Name::new("RightBarsHud"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ));
            });
    });
}
