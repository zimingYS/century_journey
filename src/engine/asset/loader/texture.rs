use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::engine::asset::loader::AssetLoader;

/// 纹理加载器。
///
/// 将纹理文件（PNG 等）加载为 Bevy `Image` 资源。
pub struct TextureLoader;

impl AssetLoader for TextureLoader {
    type Output = Image;
    fn load(&self, path: &str, asset_server: &AssetServer) -> Handle<Image> {
        asset_server.load(path.to_string())
    }
}
