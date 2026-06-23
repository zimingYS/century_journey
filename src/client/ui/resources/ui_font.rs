use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::manager::AssetManager;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct UiFont {
    pub default: Handle<Font>,
}

/// 加载字体（通过 AssetManager）
pub fn load_ui_font_system(mut ui_font: ResMut<UiFont>, mut asset: ResMut<AssetManager>) {
    let id = AssetId::default_namespace("fonts/NotoSansSC-VariableFont_wght.ttf");
    ui_font.default = asset.font(&id);
}
