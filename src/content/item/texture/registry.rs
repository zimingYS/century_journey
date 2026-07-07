use crate::engine::asset::AssetManager;
use crate::engine::asset::files::scan_dir;
use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::texture::TextureAsset;
use crate::shared::identifier::Identifier;
use bevy::prelude::*;
use std::collections::HashMap;

/// 物品独立纹理注册表
///
/// 启动时扫描 assets/textures/items/ 下所有 PNG，通过 AssetManager 加载。
#[derive(Resource, Default)]
pub struct ItemTextureRegistry {
    textures: HashMap<Identifier, TextureAsset>,
}

impl ItemTextureRegistry {
    /// 获取纹理句柄（通过标识符字符串）
    pub fn get_handle(&self, identifier: &str) -> Option<&Handle<Image>> {
        let key = Identifier::parse(identifier).unwrap_or_default();
        self.textures.get(&key).map(|a| &a.handle)
    }

    /// 获取完整 TextureAsset（通过标识符字符串）
    pub fn get(&self, identifier: &str) -> Option<&TextureAsset> {
        let key = Identifier::parse(identifier).unwrap_or_default();
        self.textures.get(&key)
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }
}

/// 启动时扫描并加载物品纹理
pub fn load_item_textures_system(
    mut commands: Commands,
    mut asset: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
) {
    let mut registry = ItemTextureRegistry::default();
    let base = asset.resolver().root_dir().join("textures/items");
    for relative in scan_dir(&base, "png") {
        let stem = relative.trim_end_matches(".png");
        let identifier = Identifier::new("century_journey", stem);
        let id = AssetId::new("century_journey", format!("textures/items/{stem}"));
        let texture_asset = asset.texture(&id, &asset_server);
        registry.textures.insert(identifier, texture_asset);
    }
    commands.insert_resource(registry);
}
