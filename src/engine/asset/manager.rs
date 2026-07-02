use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::pipeline::{AssetPipeline, AssetRequest};
use crate::engine::asset::state::AssetState;
use crate::engine::asset::texture::{TextureAsset, TextureMetadata};
use bevy::prelude::*;
use serde::de::DeserializeOwned;

/// 资源管理器 — Engine Asset 唯一公开入口（Facade）
///
/// 纹理加载走 Pipeline: Resolver→Source→Cache，
/// 自动提取 TextureMetadata (width/height)，返回 TextureAsset。
#[derive(Resource, Default)]
pub struct AssetManager {
    textures: std::collections::HashMap<String, TextureAsset>,
    pending_textures: Vec<String>,
    font_handles: std::collections::HashMap<String, Handle<Font>>,
    pending_fonts: Vec<String>,
    frame: u64,
    pub pipeline_processes: u64,
    pub pipeline_failures: u64,
}

impl AssetManager {
    /// 加载纹理 → `TextureAsset`（含 width/height 等元数据）
    pub fn texture(&mut self, id: &AssetId) -> TextureAsset {
        let key = id.to_string();
        self.ensure_texture(&key);
        self.textures.get(&key).cloned().unwrap_or_else(|| {
            TextureAsset::new(Handle::default(), TextureMetadata::default(), id.clone())
        })
    }

    /// 加载字体
    pub fn font(&mut self, id: &AssetId) -> Handle<Font> {
        let key = id.to_string();
        self.ensure_font(&key);
        self.font_handles.get(&key).cloned().unwrap_or_default()
    }

    /// 同步读取二进制文件
    pub fn read_file_bytes_sync(&self, id: &AssetId) -> Result<Vec<u8>, String> {
        let path = asset_id_to_file_path(id);
        std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))
    }

    /// 同步读取文本文件
    pub fn read_file_sync(&self, id: &AssetId) -> Result<String, String> {
        let path = asset_id_to_file_path(id);
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))
    }

    /// 同步读取并解析 JSON
    pub fn read_json_sync<T: DeserializeOwned>(&self, id: &AssetId) -> Result<T, String> {
        let content = self.read_file_sync(id)?;
        let path = asset_id_to_file_path(id);
        serde_json::from_str::<T>(&content).map_err(|e| format!("parse {}: {e}", path.display()))
    }

    /// 扫描目录批量加载 JSON
    pub fn read_json_dir_sync<T: DeserializeOwned>(&self, dir_path: &str) -> Vec<(String, T)> {
        let dir = std::path::PathBuf::from(dir_path);
        if !dir.exists() {
            return vec![];
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return vec![];
        };
        let mut results = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(value) = serde_json::from_str::<T>(&content) {
                results.push((stem.to_string(), value));
            }
        }
        results
    }

    pub fn state(&self, _id: &AssetId) -> AssetState {
        AssetState::Ready
    }
    pub fn is_ready(&self, id: &AssetId) -> bool {
        self.textures.contains_key(&id.to_string())
            || self.font_handles.contains_key(&id.to_string())
    }

    fn ensure_texture(&mut self, key: &str) {
        if !self.textures.contains_key(key) && !self.pending_textures.iter().any(|k| k == key) {
            self.pending_textures.push(key.to_string());
        }
    }
    fn ensure_font(&mut self, key: &str) {
        if !self.font_handles.contains_key(key) && !self.pending_fonts.iter().any(|k| k == key) {
            self.pending_fonts.push(key.to_string());
        }
    }

    pub(crate) fn process_pending(&mut self, asset_server: &AssetServer, pipeline: &AssetPipeline) {
        for key in std::mem::take(&mut self.pending_textures) {
            let path = asset_id_to_bevy_path(&key);
            let id = AssetId::parse(&format!("century_journey:{path}"));

            // Pipeline: Resolver → Source → Cache
            let request = AssetRequest::texture(id.clone());
            let (response, ctx) = pipeline.process(request);

            if response.success {
                self.pipeline_processes += 1;
            } else {
                self.pipeline_failures += 1;
            }

            // 从 Pipeline Context 提取图片尺寸
            let metadata = if let Some(ref bytes) = ctx.raw_bytes {
                extract_image_metadata(bytes).unwrap_or_default()
            } else {
                TextureMetadata::default()
            };

            let handle = asset_server.load(&path);
            let asset = TextureAsset::new(handle, metadata, id);
            self.textures.insert(key, asset);
        }
        for key in std::mem::take(&mut self.pending_fonts) {
            let path = asset_id_to_bevy_path(&key);
            let id = AssetId::parse(&format!("century_journey:{path}"));
            let request = AssetRequest::custom(id, "font");
            let (response, _ctx) = pipeline.process(request);
            if response.success {
                self.pipeline_processes += 1;
            } else {
                self.pipeline_failures += 1;
            }
            let handle = asset_server.load(&path);
            self.font_handles.insert(key, handle);
        }
    }
    /// 递归遍历目录，返回所有匹配扩展名的文件路径
    pub fn list_files_recursive(
        &self,
        dir_path: &str,
        extension: &str,
    ) -> Vec<(String, std::path::PathBuf)> {
        let mut results = Vec::new();
        let base = std::path::PathBuf::from(dir_path);
        Self::scan_dir_recursive(&base, &base, extension, &mut results);
        results
    }

    // 提示：这是个关联函数（没有 &self），因为纯粹是路径操作，不需要访问 AssetManager 的状态
    fn scan_dir_recursive(
        base: &std::path::Path,
        current: &std::path::Path,
        extension: &str,
        results: &mut Vec<(String, std::path::PathBuf)>,
    ) {
        let Ok(entries) = std::fs::read_dir(current) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                Self::scan_dir_recursive(base, &path, extension, results);
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some(extension) {
                continue;
            }

            let Ok(relative) = path.strip_prefix(base) else {
                continue;
            };

            let relative = relative.to_string_lossy().replace('\\', "/");

            results.push((relative, path));
        }
    }

    /// 递归扫描目录下所有 JSON 文件并解析为 T
    pub fn read_json_dir_recursive_sync<T: DeserializeOwned>(
        &self,
        dir_path: &str,
    ) -> Vec<(String, T)> {
        let mut results = Vec::new();

        for (relative_path, _) in self.list_files_recursive(dir_path, "json") {
            let relative_no_ext = relative_path
                .strip_suffix(".json")
                .unwrap_or(&relative_path);

            let asset_path = format!("{dir_path}/{relative_no_ext}");
            let id = AssetId::default_namespace(&asset_path);

            match self.read_json_sync::<T>(&id) {
                Ok(value) => results.push((asset_path, value)),
                Err(err) => warn!("Failed to load asset '{}': {}", id, err),
            }
        }

        results
    }
}

/// 从原始图片字节提取宽高
fn extract_image_metadata(bytes: &[u8]) -> Option<TextureMetadata> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    Some(TextureMetadata::from_size(rgba.width(), rgba.height()))
}

pub fn asset_manager_bridge_system(
    mut asset: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
    pipeline: Res<AssetPipeline>,
) {
    asset.process_pending(&asset_server, &pipeline);
    asset.frame += 1;
}

fn asset_id_to_bevy_path(id: &str) -> String {
    let cleaned = id.split(':').last().unwrap_or(id);
    if cleaned.contains('.') {
        cleaned.to_string()
    } else {
        format!("{cleaned}.png")
    }
}

fn asset_id_to_file_path(id: &AssetId) -> std::path::PathBuf {
    let s = id.to_string();
    let cleaned = s.split(':').last().unwrap_or(&s);
    let path = std::path::PathBuf::from("assets").join(cleaned);
    if path.extension().is_some() {
        path
    } else {
        path.with_extension("json")
    }
}
