//! 校验矿脉参数与方块引用。

use super::super::ContentCheckReport;
use crate::content::ore_vein::definition::OreVeinDefinition;

/// 校验全部矿脉定义及其跨方块引用。
pub(in crate::content::validation) fn validate_ore_veins(
    veins: &[(String, OreVeinDefinition)],
    block_ids: &std::collections::HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, vein) in veins {
        if !block_ids.contains(&vein.block.to_string()) {
            report
                .errors
                .push(format!("{path}:block: unknown block {}", vein.block));
        }
        if vein.block.namespace() == "century_journey" && vein.block.path() == "air" {
            report.errors.push(format!(
                "{path}:block: ore vein cannot use century_journey:air"
            ));
        }
        if vein.min_y > vein.max_y {
            report
                .errors
                .push(format!("{path}:min_y: must be <= max_y"));
        }
        if vein.scale <= 0.0 {
            report
                .errors
                .push(format!("{path}:scale: must be greater than zero"));
        }
        if !(-1.0..=1.0).contains(&vein.threshold) {
            report
                .errors
                .push(format!("{path}:threshold: expected value in [-1, 1]"));
        }
    }
}
