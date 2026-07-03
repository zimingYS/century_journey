use crate::engine::asset::texture::TextureAsset;
use crate::engine::asset::texture::TextureMetadata;
use bevy::image::ImageLoaderSettings;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 物品独立纹理注册表
///
/// 启动时扫描 assets/textures/items/ 下所有 PNG，通过 AssetManager 加载。
#[derive(Resource, Default)]
pub struct ItemTextureRegistry {
    textures: HashMap<String, TextureAsset>,
}

impl ItemTextureRegistry {
    /// 获取纹理句柄（向后兼容）
    pub fn get_handle(&self, identifier: &str) -> Option<&Handle<Image>> {
        self.textures.get(identifier).map(|a| &a.handle)
    }

    /// 获取完整 TextureAsset
    pub fn get(&self, identifier: &str) -> Option<&TextureAsset> {
        self.textures.get(identifier)
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }
}

/// 启动时扫描并加载物品纹理
/// 使用 AssetServer 直接获取即时 Handle（启动阶段），
/// 纹理元数据从文件自动提取。
pub fn load_item_textures_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dir = PathBuf::from("assets/textures/items");
    if !dir.exists() {
        info!("[ItemTexture] textures/items/ 目录不存在，跳过加载");
        commands.insert_resource(ItemTextureRegistry::default());
        return;
    }

    let Ok(entries) = fs::read_dir(&dir) else {
        commands.insert_resource(ItemTextureRegistry::default());
        return;
    };

    let mut registry = ItemTextureRegistry::default();
    let mut loaded = 0usize;
    let mut missing = 0usize;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("png") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let identifier = format!("century_journey:{}", stem);

        if path.metadata().map(|m| m.len() == 0).unwrap_or(true) {
            warn!("[ItemTexture] 空文件跳过: {}", path.display());
            missing += 1;
            continue;
        }

        let asset_path = format!("textures/items/{stem}.png");
        // 通过 AssetServer 直接获取即时 Handle（启动阶段）
        let handle: Handle<Image> = asset_server
            .load_builder()
            .with_settings(|s: &mut ImageLoaderSettings| {
                s.sampler = ImageSampler::nearest();
            })
            .load(asset_path);

        // 从文件提取元数据
        let metadata = fs::read(&path)
            .ok()
            .and_then(|bytes| {
                image::load_from_memory(&bytes).ok().map(|img| {
                    let rgba = img.to_rgba8();
                    TextureMetadata::from_size(rgba.width(), rgba.height())
                })
            })
            .unwrap_or_default();

        let texture_asset = TextureAsset::new(
            handle,
            metadata,
            crate::engine::asset::identifier::asset_id_parse(&identifier),
        );

        registry.textures.insert(identifier.clone(), texture_asset);
        loaded += 1;
        info!("[ItemTexture] 已加载: {}", identifier);
    }

    if missing > 0 {
        warn!("[ItemTexture] {} 个纹理缺失或为空", missing);
    }

    info!(
        "[ItemTexture] 物品纹理注册完成: {} 个 ({} 缺失)",
        loaded, missing
    );

    commands.insert_resource(registry);
}
