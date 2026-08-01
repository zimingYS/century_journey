//! 校验树种参数、方块引用和树苗到树种的一对一关系。

use super::super::ContentCheckReport;
use crate::content::block::definition::BlockProperty;
use crate::content::vegetation::definition::{TreeSizeRange, TreeSpeciesDefinition};
use std::collections::{HashMap, HashSet};

/// 校验全部树种定义及其跨方块引用。
pub(in crate::content::validation) fn validate_tree_species(
    species: &[(String, TreeSpeciesDefinition)],
    blocks: &[(String, BlockProperty)],
    block_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    let block_definitions = blocks
        .iter()
        .map(|(_, block)| (block.identifier.to_string(), block))
        .collect::<HashMap<_, _>>();
    let mut sapling_owners = HashMap::<String, String>::new();

    for (path, definition) in species {
        for (field, block) in [
            ("sapling_block", &definition.sapling_block),
            ("trunk_block", &definition.trunk_block),
            ("leaves_block", &definition.leaves_block),
        ] {
            if !block_ids.contains(&block.to_string()) {
                report
                    .errors
                    .push(format!("{path}:{field}: unknown block {block}"));
            }
        }

        let sapling_key = definition.sapling_block.to_string();
        if [
            &definition.sapling_block,
            &definition.trunk_block,
            &definition.leaves_block,
        ]
        .iter()
        .any(|identifier| identifier.namespace() == "century_journey" && identifier.path() == "air")
        {
            report.errors.push(format!(
                "{path}:block references: tree species cannot use century_journey:air"
            ));
        }
        if definition.sapling_block == definition.trunk_block
            || definition.sapling_block == definition.leaves_block
        {
            report.errors.push(format!(
                "{path}:sapling_block: sapling must differ from trunk and leaves"
            ));
        }
        if let Some(previous) = sapling_owners.insert(sapling_key.clone(), path.clone()) {
            report.errors.push(format!(
                "{path}:sapling_block: {sapling_key} is already owned by {previous}"
            ));
        }
        if let Some(sapling) = block_definitions.get(&sapling_key)
            && sapling.placement.required_support_tag.is_none()
        {
            report.errors.push(format!(
                "{path}:sapling_block: {sapling_key} must declare placement.required_support_tag"
            ));
        }

        for (field, duration) in [
            (
                "growth.sapling_duration_game_minutes",
                definition.growth.sapling_duration_game_minutes,
            ),
            (
                "growth.young_duration_game_minutes",
                definition.growth.young_duration_game_minutes,
            ),
            (
                "growth.retry_interval_game_minutes",
                definition.growth.retry_interval_game_minutes,
            ),
        ] {
            if duration == 0 {
                report
                    .errors
                    .push(format!("{path}:{field}: must be greater than zero"));
            }
        }

        if let Some(young) = definition.young_blueprint {
            validate_size_range(
                path,
                "young_blueprint.trunk_height",
                young.trunk_height,
                64,
                report,
            );
            validate_size_range(
                path,
                "young_blueprint.crown_radius",
                young.crown_radius,
                16,
                report,
            );
        }

        validate_size_range(
            path,
            "blueprint.trunk_height",
            definition.blueprint.trunk_height,
            64,
            report,
        );
        validate_size_range(
            path,
            "blueprint.crown_radius",
            definition.blueprint.crown_radius,
            16,
            report,
        );
    }
}

fn validate_size_range(
    path: &str,
    field: &str,
    range: TreeSizeRange,
    maximum: u8,
    report: &mut ContentCheckReport,
) {
    if range.min == 0 || range.min > range.max || range.max > maximum {
        report.errors.push(format!(
            "{path}:{field}: expected 1 <= min <= max <= {maximum}"
        ));
    }
}
