use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::location::AssetLocation;
use bevy::prelude::Resource;
use std::path::{Path, PathBuf};

/// 资源解析器 —— 唯一负责 `AssetId -> 路径` 转换的地方。
///
/// 现阶段只有一个根目录（`assets/`）。以后如果要支持 Mod / 资源包覆盖，
/// 优先考虑用 Bevy 原生的多 `AssetSource`（`app.register_asset_source`），
/// 而不是在这里重新实现一套优先级链。
#[derive(Resource, Clone)]
pub struct AssetResolver {
    root: PathBuf,
}

impl AssetResolver {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root
    }

    /// 解析为带扩展名的资源位置（纹理 / 字体等 Handle 资源用）。
    /// 若 `id.path()` 已经带扩展名则原样使用，否则拼接 `default_extension`。
    pub fn resolve(&self, id: &AssetId, default_extension: &str) -> AssetLocation {
        let path = id.path();
        let relative = if path.contains('.') {
            path.to_string()
        } else {
            format!("{path}.{default_extension}")
        };
        AssetLocation::new(relative.clone(), self.root.join(relative))
    }

    /// 解析为裸路径（不补扩展名），配置文件场景用。
    pub fn resolve_raw(&self, id: &AssetId) -> AssetLocation {
        let relative = id.path().to_string();
        AssetLocation::new(relative.clone(), self.root.join(relative))
    }
}

impl Default for AssetResolver {
    fn default() -> Self {
        Self::new("assets")
    }
}
