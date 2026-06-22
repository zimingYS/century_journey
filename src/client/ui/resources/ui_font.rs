use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct UiFont {
    pub default: Handle<Font>,
}

/// 加载字体
pub fn load_ui_font_system(mut ui_font: ResMut<UiFont>, asset_server: Res<AssetServer>) {
    ui_font.default = asset_server.load("fonts/NotoSansSC-VariableFont_wght.ttf");
}
