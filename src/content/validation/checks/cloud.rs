//! 云定义参数范围、排序约束与可扩展运行时入口校验。

use super::super::ContentCheckReport;
use crate::content::cloud::definition::CloudDefinition;
use std::collections::HashSet;

/// 校验云场标识、密度、层数、风场和近景云片参数。
pub(in crate::content::validation) fn validate_clouds(
    clouds: &[(String, CloudDefinition)],
    report: &mut ContentCheckReport,
) {
    let mut identifiers = HashSet::new();
    for (path, cloud) in clouds {
        if !identifiers.insert(cloud.identifier.to_string()) {
            report.errors.push(format!(
                "{path}:identifier: duplicate cloud identifier {}",
                cloud.identifier
            ));
        }
        if !cloud.density.is_finite() || !(0.0..=1.0).contains(&cloud.density) {
            report
                .errors
                .push(format!("{path}:density: must be finite and within 0..=1"));
        }
        if cloud.layers.is_empty() || cloud.layers.len() > 8 {
            report
                .errors
                .push(format!("{path}:layers: must contain 1 to 8 layers"));
        }

        let mut previous_height = None;
        for (index, layer) in cloud.layers.iter().enumerate() {
            let prefix = format!("{path}:layers[{index}]");
            if !layer.height.is_finite() || !(0.0..=1024.0).contains(&layer.height) {
                report.errors.push(format!(
                    "{prefix}.height: must be finite and within 0..=1024"
                ));
            }
            if !layer.size.is_finite() || !(32.0..=4096.0).contains(&layer.size) {
                report.errors.push(format!(
                    "{prefix}.size: must be finite and within 32..=4096"
                ));
            }
            if !layer.speed.is_finite() || !(0.0..=128.0).contains(&layer.speed) {
                report
                    .errors
                    .push(format!("{prefix}.speed: must be finite and within 0..=128"));
            }
            let [wind_x, wind_z] = layer.wind_direction;
            if !wind_x.is_finite()
                || !wind_z.is_finite()
                || (wind_x * wind_x + wind_z * wind_z) < 0.0001
            {
                report.errors.push(format!(
                    "{prefix}.wind_direction: must be finite and non-zero"
                ));
            }
            if let Some(previous) = previous_height
                && layer.height < previous
            {
                report.errors.push(format!(
                    "{prefix}.height: layers must be ordered from low to high"
                ));
            }
            previous_height = Some(layer.height);
            if !(0.0..=1.0).contains(&layer.opacity) || !layer.opacity.is_finite() {
                report
                    .errors
                    .push(format!("{prefix}.opacity: must be finite and within 0..=1"));
            }
            for tint in [&layer.tint_day, &layer.tint_night, &layer.tint_sunset] {
                if tint
                    .iter()
                    .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
                {
                    report.errors.push(format!(
                        "{prefix}: tint values must be finite and within 0..=1"
                    ));
                    break;
                }
            }
        }

        let patches = &cloud.patches;
        if patches.enabled {
            if patches.count > 128 {
                report
                    .errors
                    .push(format!("{path}:patches.count: must not exceed 128"));
            }
            if !patches.spawn_radius.is_finite() || !(1.0..=2048.0).contains(&patches.spawn_radius)
            {
                report.errors.push(format!(
                    "{path}:patches.spawn_radius: must be finite and within 1..=2048"
                ));
            }
            if !patches.scale_min.is_finite()
                || !patches.scale_max.is_finite()
                || !(1.0..=512.0).contains(&patches.scale_min)
                || !(1.0..=512.0).contains(&patches.scale_max)
                || patches.scale_min > patches.scale_max
            {
                report.errors.push(format!(
                    "{path}:patches.scale_min/scale_max: must be ordered within 1..=512"
                ));
            }
            if !patches.opacity.is_finite() || !(0.0..=1.0).contains(&patches.opacity) {
                report.errors.push(format!(
                    "{path}:patches.opacity: must be finite and within 0..=1"
                ));
            }
        }
    }
    if clouds.is_empty() {
        report
            .errors
            .push("definitions/clouds:directory: at least one cloud definition is required".into());
    }
}
