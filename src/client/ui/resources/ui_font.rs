use crate::engine::asset::identifier::asset_id;
use crate::engine::asset::manager::AssetManager;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct UiFont {
    pub default: Handle<Font>,
}

/// 加载项目 UI 字体（通过 AssetManager）。
pub fn load_ui_font_system(
    mut ui_font: ResMut<UiFont>,
    mut asset: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
) {
    let id = asset_id("fonts/fusion-pixel/fusion-pixel-10px-monospaced-zh_hans.otf");
    ui_font.default = asset.font(&id, &asset_server);
}
