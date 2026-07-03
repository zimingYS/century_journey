use crate::content::biome::definition::BiomeTerrainParams;
use crate::content::biome::registry::BiomeRegistry;

/// 根据温度/湿度选择最佳匹配的生物群系索引。
pub fn select_biome(biome_registry: &BiomeRegistry, temperature: f64, humidity: f64) -> u8 {
    let mut best_index = 0u8;
    let mut best_score = f64::MAX;

    for (idx, biome) in biome_registry.biomes.iter().enumerate() {
        let temp_center = (biome.temperature_range.0 + biome.temperature_range.1) * 0.5;
        let humid_center = (biome.humidity_range.0 + biome.humidity_range.1) * 0.5;
        let temp_dist = (temperature - temp_center).abs();
        let humid_dist = (humidity - humid_center).abs();
        let score = temp_dist * 1.2 + humid_dist;

        if score < best_score {
            best_score = score;
            best_index = idx as u8;
        }
    }
    best_index
}

/// 基于所有群系的高斯加权混合，计算混合地形参数。
pub fn blend_terrain_params(
    biome_registry: &BiomeRegistry,
    temperature: f64,
    humidity: f64,
) -> BiomeTerrainParams {
    let mut total_weight = 0.0;
    let mut blended = BiomeTerrainParams {
        base_height: 0.0,
        height_amplitude: 0.0,
        roughness: 0.0,
    };

    for biome in &biome_registry.biomes {
        let temp_center = (biome.temperature_range.0 + biome.temperature_range.1) * 0.5;
        let humid_center = (biome.humidity_range.0 + biome.humidity_range.1) * 0.5;
        let temp_dist = (temperature - temp_center).abs();
        let humid_dist = (humidity - humid_center).abs();
        let score = temp_dist * 1.2 + humid_dist;
        let weight = (-score * score * 6.0).exp();

        blended.base_height += biome.terrain.base_height * weight;
        blended.height_amplitude += biome.terrain.height_amplitude * weight;
        blended.roughness += biome.terrain.roughness * weight;
        total_weight += weight;
    }

    if total_weight > 0.0 {
        blended.base_height /= total_weight;
        blended.height_amplitude /= total_weight;
        blended.roughness /= total_weight;
    }
    blended
}
