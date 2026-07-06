use crate::engine::asset::location::AssetLocation;
use crate::engine::asset::manager::AssetManager;
use bevy::asset::AssetServer;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

/// 资源加载管道 —— 位于 `AssetLocation` 和 `AssetServer` 之间。
///
/// 只做两件事：
/// 1. 按资源类型应用正确的加载参数（比如像素画用最近邻采样）；
/// 2. 加载完成后的轻量后处理（纹理宽高回填，见 [`sync_texture_metadata_system`]）。
///
/// 真正的 IO / 解码 / 缓存 / 热重载全部是 Bevy `AssetServer` + `AssetSource` 的职责，
/// Pipeline 不重复实现，只在其上加一层业务规则。
pub struct AssetPipeline;

impl AssetPipeline {
    /// 加载像素风格纹理：最近邻采样，无 mipmap。
    pub fn load_pixel_texture(
        asset_server: &AssetServer,
        location: &AssetLocation,
    ) -> Handle<Image> {
        asset_server
            .load_builder()
            .with_settings(|s: &mut ImageLoaderSettings| {
                s.sampler = ImageSampler::nearest();
            })
            .load(location.relative_path.clone())
    }

    /// 其余类型直接透传给 AssetServer，不加额外规则。
    pub fn load<T: Asset>(asset_server: &AssetServer, location: &AssetLocation) -> Handle<T> {
        asset_server.load(location.relative_path.clone())
    }
}

/// 每帧检查还未拿到真实宽高的纹理，一旦 Bevy 完成解码（`Assets<Image>` 里能查到）
/// 就把宽高回填到 `AssetManager` 缓存的 `TextureAsset` 里。
///
/// 这是纹理宽高唯一的提取位置——不再有第二处手动 `image::load_from_memory` 去解码。
pub fn sync_texture_metadata_system(mut manager: ResMut<AssetManager>, images: Res<Assets<Image>>) {
    manager.sync_pending_texture_metadata(&images);
}
