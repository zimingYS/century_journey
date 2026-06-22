use bevy::image::ImageLoaderSettings;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 物品独立纹理注册表
///
/// 启动时扫描 assets/textures/items/ 下所有 PNG，加载为独立 Handle<Image>。
/// 每个纹理以 "century_journey:stem" 为 key。
#[derive(Resource, Default)]
pub struct ItemTextureRegistry {
    textures: HashMap<String, Handle<Image>>,
}

impl ItemTextureRegistry {
    /// 通过 identifier 获取纹理 Handle
    pub fn get(&self, identifier: &str) -> Option<&Handle<Image>> {
        self.textures.get(identifier)
    }

    /// 已加载的纹理数量
    pub fn len(&self) -> usize {
        self.textures.len()
    }
}

/// 启动时扫描并加载物品纹理
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

        let asset_path = format!("textures/items/{}.png", stem);
        let handle: Handle<Image> =
            asset_server.load_with_settings(&asset_path, |s: &mut ImageLoaderSettings| {
                s.sampler = ImageSampler::nearest();
            });

        registry.textures.insert(identifier.clone(), handle);
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
