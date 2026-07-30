//! 从数据资产加载并解析生物群系定义。

use crate::content::biome::definition::BiomeDefinition;
use crate::content::format::load_versioned_json_dir;
use crate::engine::asset::{AssetFiles, AssetManager};

/// 从资产管理器加载全部生物群系定义。
pub fn load_biome_definitions(asset: &AssetManager) -> Vec<BiomeDefinition> {
    let files = AssetFiles::new(asset.resolver());
    load_versioned_json_dir::<BiomeDefinition>(&files, "definitions/biomes")
        .into_iter()
        .map(|(_, biome)| biome)
        .collect()
}
