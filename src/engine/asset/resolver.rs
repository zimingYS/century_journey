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
    content_roots: Vec<PathBuf>,
}

impl AssetResolver {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            content_roots: vec![root.clone()],
            root,
        }
    }

    /// 构建内容来源栈。来源按低到高优先级排列，后声明的来源覆盖同路径文件。
    pub fn with_content_overrides(
        root: impl Into<PathBuf>,
        overrides: impl IntoIterator<Item = PathBuf>,
    ) -> Self {
        let root = root.into();
        let mut content_roots = vec![root.clone()];
        content_roots.extend(overrides);
        Self {
            root,
            content_roots,
        }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root
    }

    pub fn content_roots(&self) -> &[PathBuf] {
        &self.content_roots
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

    /// 解析同步内容文件；同一路径由优先级最高且实际存在的来源提供。
    pub fn resolve_content(&self, id: &AssetId, default_extension: &str) -> AssetLocation {
        let path = id.path();
        let relative = if path.contains('.') {
            path.to_string()
        } else {
            format!("{path}.{default_extension}")
        };
        let full_path = self
            .content_roots
            .iter()
            .rev()
            .map(|root| root.join(&relative))
            .find(|candidate| candidate.is_file())
            .unwrap_or_else(|| self.root.join(&relative));
        AssetLocation::new(relative, full_path)
    }
}

impl Default for AssetResolver {
    fn default() -> Self {
        let root = std::env::var_os("CJ_ASSET_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("assets"));
        let overrides = std::env::var_os("CJ_CONTENT_OVERRIDES")
            .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
            .unwrap_or_default();
        Self::with_content_overrides(root, overrides)
    }
}
