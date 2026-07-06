use crate::engine::asset::identifier::{AssetId, asset_id};
use crate::engine::asset::resolver::AssetResolver;
use serde::de::DeserializeOwned;
use std::path::Path;

/// 同步文件/配置读取工具 —— 与 `AssetManager` 完全分离的 File API。
///
/// `AssetManager` 面向需要 `Handle<T>` 的资源（纹理、字体……），异步、走 Bevy AssetServer；
/// `AssetFiles` 面向配置/数据定义文件（JSON 等），同步、直接读磁盘，用完即弃。
/// 两者共用同一个 [`AssetResolver`]，路径转换只有一份实现。
pub struct AssetFiles<'a> {
    resolver: &'a AssetResolver,
}

impl<'a> AssetFiles<'a> {
    pub fn new(resolver: &'a AssetResolver) -> Self {
        Self { resolver }
    }

    pub fn read_bytes(&self, id: &AssetId) -> Result<Vec<u8>, String> {
        let location = self.resolver.resolve_raw(id);
        std::fs::read(&location.full_path)
            .map_err(|e| format!("read {}: {e}", location.full_path.display()))
    }

    pub fn read_string(&self, id: &AssetId) -> Result<String, String> {
        let location = self.resolver.resolve(id, "txt");
        std::fs::read_to_string(&location.full_path)
            .map_err(|e| format!("read {}: {e}", location.full_path.display()))
    }

    pub fn read_json<T: DeserializeOwned>(&self, id: &AssetId) -> Result<T, String> {
        let location = self.resolver.resolve(id, "json");
        let content = std::fs::read_to_string(&location.full_path)
            .map_err(|e| format!("read {}: {e}", location.full_path.display()))?;
        serde_json::from_str(&content).map_err(|e| format!("parse {}: {e}", id))
    }

    /// 递归扫描逻辑目录（相对于 assets 根目录，如 `"definitions/blocks"`）下所有 JSON 文件并解析为 `T`。
    /// 取代原来 `read_json_dir_sync` + `read_json_dir_recursive_sync` 两个重复实现。
    pub fn read_json_dir<T: DeserializeOwned>(&self, dir_path: &str) -> Vec<(String, T)> {
        let base = self.resolver.root_dir().join(dir_path);
        let mut results = Vec::new();
        for relative in scan_dir(&base, "json") {
            let no_ext = relative.strip_suffix(".json").unwrap_or(&relative);
            let asset_path = format!("{dir_path}/{no_ext}");
            let id = asset_id(&asset_path);
            match self.read_json::<T>(&id) {
                Ok(value) => results.push((asset_path, value)),
                Err(err) => bevy::log::warn!("Failed to load asset '{}': {}", id, err),
            }
        }
        results
    }
}

/// 递归遍历目录，返回相对路径列表。纯路径操作，不属于任何 Manager 的方法，
/// 是独立工具函数——原来挂在 `AssetManager::list_files_recursive` 上是错的。
pub fn scan_dir(base: &Path, extension: &str) -> Vec<String> {
    fn walk(base: &Path, current: &Path, extension: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(current) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, extension, out);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some(extension) {
                continue;
            }
            if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    if base.exists() {
        walk(base, base, extension, &mut out);
    }
    out
}
