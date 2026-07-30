//! 方块定义、模型、状态和纹理引用校验。

use super::super::{ContentCheckReport, paths::content_file_exists};
use crate::content::block::definition::BlockProperty;
use crate::content::block::model::BlockModel;
use crate::engine::asset::AssetResolver;
use std::collections::HashSet;

/// 校验方块标识符、状态、模型与纹理引用。
pub(in crate::content::validation) fn validate_blocks(
    resolver: &AssetResolver,
    blocks: &[(String, BlockProperty)],
    block_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, block) in blocks {
        if !block.hardness.is_finite() || block.hardness < 0.0 {
            report
                .errors
                .push(format!("{path}:hardness: must be finite and non-negative"));
        }
        if block.light_emission > 15 {
            report
                .errors
                .push(format!("{path}:light_emission: must be <= 15"));
        }
        if !block.light_transmission.is_finite() || !(0.0..=1.0).contains(&block.light_transmission)
        {
            report.errors.push(format!(
                "{path}:light_transmission: must be finite and within 0..=1"
            ));
        }
        for face in 0..6 {
            let texture = block.textures.get_face_texture(face);
            if !content_file_exists(resolver, texture) {
                report.errors.push(format!(
                    "{path}:textures.{face}: missing block texture {texture}"
                ));
            }
        }
        if let Some(drop) = &block.drop_identifier
            && !block_ids.contains(&drop.to_string())
        {
            report
                .errors
                .push(format!("{path}:drop_identifier: unknown block {drop}"));
        }
        for (field, volume) in [
            ("sound.break_volume", block.sound.break_volume),
            ("sound.place_volume", block.sound.place_volume),
            ("sound.step_volume", block.sound.step_volume),
        ] {
            if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
                report
                    .errors
                    .push(format!("{path}:{field}: must be finite and within 0..=1"));
            }
        }
        validate_block_model(path, &block.model.model, report);
        let mut property_names = HashSet::new();
        let mut state_count = 1usize;
        for (index, property) in block.states.properties.iter().enumerate() {
            let field = format!("states.properties[{index}]");
            if property.name.is_empty() || !property_names.insert(property.name.as_str()) {
                report
                    .errors
                    .push(format!("{path}:{field}.name: must be non-empty and unique"));
            }
            if property.values.is_empty() {
                report
                    .errors
                    .push(format!("{path}:{field}.values: must not be empty"));
                continue;
            }
            if property.default_index >= property.values.len() {
                report.errors.push(format!(
                    "{path}:{field}.default_index: exceeds values length {}",
                    property.values.len()
                ));
            }
            let unique_values = property.values.iter().collect::<HashSet<_>>();
            if unique_values.len() != property.values.len() {
                report
                    .errors
                    .push(format!("{path}:{field}.values: contains duplicates"));
            }
            state_count = state_count.saturating_mul(property.values.len());
        }
        if state_count > u16::MAX as usize + 1 {
            report.errors.push(format!(
                "{path}:states.properties: {state_count} combinations exceed the u16 state space"
            ));
        }
    }
}

/// 校验单个方块模型的几何参数和贴图索引。
fn validate_block_model(path: &str, model: &BlockModel, report: &mut ContentCheckReport) {
    match model {
        BlockModel::Slab { thickness }
            if !thickness.is_finite() || !(0.0..=1.0).contains(thickness) =>
        {
            report.errors.push(format!(
                "{path}:model.model.thickness: must be finite and within 0..=1"
            ));
        }
        BlockModel::Custom { faces } => {
            for (index, face) in faces.iter().enumerate() {
                if face
                    .vertices
                    .iter()
                    .flatten()
                    .any(|coordinate| !coordinate.is_finite())
                    || face.normal.iter().any(|coordinate| !coordinate.is_finite())
                {
                    report.errors.push(format!(
                        "{path}:model.model.faces[{index}]: vertices and normal must be finite"
                    ));
                }
                if !face.ambient_occlusion.is_finite()
                    || !(0.0..=1.0).contains(&face.ambient_occlusion)
                {
                    report.errors.push(format!(
                        "{path}:model.model.faces[{index}].ambient_occlusion: must be finite and within 0..=1"
                    ));
                }
            }
        }
        _ => {}
    }
}
