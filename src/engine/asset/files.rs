use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::resolver::AssetResolver;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ResolvedAssetFile {
    pub asset_path: String,
    pub full_path: PathBuf,
    pub source_index: usize,
}

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
        let location = self.resolver.resolve_content(id, "json");
        let content = std::fs::read_to_string(&location.full_path)
            .map_err(|e| format!("read {}: {e}", location.full_path.display()))?;
        serde_json::from_str(&content).map_err(|e| format!("parse {}: {e}", id))
    }

    /// 递归扫描逻辑目录（相对于 assets 根目录，如 `"definitions/blocks"`）下所有 JSON 文件并解析为 `T`。
    /// 取代原来 `read_json_dir_sync` + `read_json_dir_recursive_sync` 两个重复实现。
    pub fn read_json_dir<T: DeserializeOwned>(&self, dir_path: &str) -> Vec<(String, T)> {
        let mut results = Vec::new();
        for result in self.read_json_dir_results::<T>(dir_path) {
            match result {
                Ok(value) => results.push(value),
                Err(err) => bevy::log::warn!("Failed to load asset: {err}"),
            }
        }
        results
    }

    /// 返回覆盖合并后的文件列表。后声明内容来源覆盖同相对路径文件。
    pub fn resolved_files(&self, dir_path: &str, extension: &str) -> Vec<ResolvedAssetFile> {
        let mut merged = BTreeMap::new();
        for (source_index, root) in self.resolver.content_roots().iter().enumerate() {
            let base = root.join(dir_path);
            for relative in scan_dir(&base, extension) {
                let full_path = base.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
                let no_ext = relative
                    .strip_suffix(&format!(".{extension}"))
                    .unwrap_or(&relative);
                let asset_path = format!("{dir_path}/{no_ext}");
                merged.insert(
                    asset_path.clone(),
                    ResolvedAssetFile {
                        asset_path,
                        full_path,
                        source_index,
                    },
                );
            }
        }
        merged.into_values().collect()
    }

    /// 严格读取目录，保留每个文件的解析错误供内容检查命令汇总。
    pub fn read_json_dir_results<T: DeserializeOwned>(
        &self,
        dir_path: &str,
    ) -> Vec<Result<(String, T), String>> {
        self.resolved_files(dir_path, "json")
            .into_iter()
            .map(|file| {
                let content = std::fs::read_to_string(&file.full_path)
                    .map_err(|error| format!("read {}: {error}", file.full_path.display()))?;
                let value = serde_json::from_str::<T>(&content)
                    .map_err(|error| format!("parse {}: {error}", file.full_path.display()))?;
                Ok((file.asset_path, value))
            })
            .collect()
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
    out.sort();
    out
}
