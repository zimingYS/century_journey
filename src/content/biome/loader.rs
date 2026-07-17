use crate::content::biome::definition::BiomeDefinition;
use crate::content::format::load_versioned_json_dir;
use crate::engine::asset::{AssetFiles, AssetManager};

pub fn load_biome_definitions(asset: &AssetManager) -> Vec<BiomeDefinition> {
    let files = AssetFiles::new(asset.resolver());
    load_versioned_json_dir::<BiomeDefinition>(&files, "definitions/biomes")
        .into_iter()
        .map(|(_, biome)| biome)
        .collect()
}
