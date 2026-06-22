use crate::engine::asset::source::manager::SourceManager;
use crate::engine::asset::source::priority::SourcePriority;
use crate::engine::asset::source::resource_pack::metadata::ResourcePackMetadata;
use crate::engine::asset::source::resource_pack::source::ResourcePackSource;
use bevy::prelude::*;

/// 资源包管理器
///
/// 负责扫描 resourcepacks/ 目录、加载清单、排序、启用/禁用。
#[derive(Resource, Default)]
pub struct ResourcePackManager {
    packs: Vec<ResourcePackMetadata>,
}

impl ResourcePackManager {
    /// 扫描 resourcepacks/ 目录
    ///
    /// 每个子目录视为一个资源包。资源包目录必须包含 assets/ 子目录。
    pub fn scan(&mut self, root: &str) -> Vec<&ResourcePackMetadata> {
        self.packs.clear();
        let Ok(entries) = std::fs::read_dir(root) else {
            return vec![];
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let assets_path = path.join("assets");
                if assets_path.exists() && assets_path.is_dir() {
                    let id = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let meta = ResourcePackMetadata::new(id, path.to_str().unwrap_or(""));
                    self.packs.push(meta);
                }
            }
        }
        self.packs()
    }

    /// 获取所有发现的资源包
    pub fn packs(&self) -> Vec<&ResourcePackMetadata> {
        self.packs.iter().collect()
    }

    /// 启用指定资源包并注册到 SourceManager
    pub fn enable(&self, pack_id: &str, source_manager: &mut SourceManager) -> Result<(), String> {
        let pack = self
            .packs
            .iter()
            .find(|p| p.id == pack_id)
            .ok_or_else(|| format!("pack not found: {pack_id}"))?;
        let source =
            ResourcePackSource::new(&pack.id, &pack.root_path, SourcePriority::UserResourcePack);
        source_manager.add(source);
        Ok(())
    }

    /// 禁用指定资源包
    pub fn disable(&self, pack_id: &str, source_manager: &mut SourceManager) {
        source_manager.remove_by_name(&format!("ResourcePack:{}", pack_id));
    }

    /// 默认资源包（使用 assets/ 目录作为基础）
    pub fn load_default(source_manager: &mut SourceManager) {
        let default = ResourcePackSource::new("default", ".", SourcePriority::DefaultResourcePack);
        source_manager.add(default);
    }
}
