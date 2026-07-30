//! 生物群系参数范围、稳定顺序和方块引用校验。

use super::super::ContentCheckReport;
use crate::content::biome::BiomeDefinition;
use std::collections::HashSet;

/// 校验生物群系参数范围、排序约束与方块引用。
pub(in crate::content::validation) fn validate_biomes(
    biomes: &[(String, BiomeDefinition)],
    block_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    let mut ids = HashSet::new();
    let mut orders = HashSet::new();
    for (path, biome) in biomes {
        if !ids.insert(biome.identifier.to_string()) {
            report.errors.push(format!(
                "{path}:identifier: duplicate biome identifier {}",
                biome.identifier
            ));
        }
        if !orders.insert(biome.generation_order) {
            report.errors.push(format!(
                "{path}:generation_order: duplicate value {}",
                biome.generation_order
            ));
        }
        validate_unit_range(path, "temperature_range", biome.temperature_range, report);
        validate_unit_range(path, "humidity_range", biome.humidity_range, report);
        if !(0.0..=1.0).contains(&biome.tree_density) || !biome.tree_density.is_finite() {
            report
                .errors
                .push(format!("{path}:tree_density: must be within 0..=1"));
        }
        if !biome.terrain.base_height.is_finite() {
            report
                .errors
                .push(format!("{path}:terrain.base_height: must be finite"));
        }
        if !biome.terrain.height_amplitude.is_finite() || biome.terrain.height_amplitude < 0.0 {
            report.errors.push(format!(
                "{path}:terrain.height_amplitude: must be finite and non-negative"
            ));
        }
        if !biome.terrain.roughness.is_finite() || biome.terrain.roughness < 0.0 {
            report.errors.push(format!(
                "{path}:terrain.roughness: must be finite and non-negative"
            ));
        }
        for (field, block) in [
            ("surface_block", &biome.surface_block),
            ("subsurface_block", &biome.subsurface_block),
            ("beach_block", &biome.beach_block),
        ] {
            if !block_ids.contains(&block.to_string()) {
                report
                    .errors
                    .push(format!("{path}:{field}: unknown block {block}"));
            }
        }
    }
    if biomes.is_empty() {
        report
            .errors
            .push("definitions/biomes:directory: at least one biome is required".into());
    }
}

/// 校验要求落在闭区间 0 到 1 内的数值范围。
fn validate_unit_range(
    path: &str,
    field: &str,
    range: (f64, f64),
    report: &mut ContentCheckReport,
) {
    if !range.0.is_finite()
        || !range.1.is_finite()
        || range.0 < 0.0
        || range.1 > 1.0
        || range.0 > range.1
    {
        report
            .errors
            .push(format!("{path}:{field}: must be ordered within 0..=1"));
    }
}
