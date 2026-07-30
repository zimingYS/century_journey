//! 方块掉落表目标、物品引用和数量概率校验。

use super::super::{ContentCheckReport, paths::definition_id};
use crate::content::loot::table::LootTable;
use std::collections::HashSet;

/// 校验方块掉落表目标、物品引用与数量概率。
pub(in crate::content::validation) fn validate_loot(
    tables: &[(String, LootTable)],
    block_ids: &HashSet<String>,
    item_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, table) in tables {
        let block_id = definition_id(path, "definitions/loot/blocks/");
        if !block_ids.contains(&block_id) {
            report.errors.push(format!(
                "{path}:path: loot table targets unknown block {block_id}"
            ));
        }
        for (index, entry) in table.entries.iter().enumerate() {
            if !item_ids.contains(&entry.item.to_string()) {
                report.errors.push(format!(
                    "{path}:entries[{index}].item: unknown item {}",
                    entry.item
                ));
            }
            if entry.min_count > entry.max_count {
                report.errors.push(format!(
                    "{path}:entries[{index}].min_count: exceeds max_count"
                ));
            }
            if entry.max_count == 0 {
                report.errors.push(format!(
                    "{path}:entries[{index}].max_count: must be positive"
                ));
            }
            if !(0.0..=1.0).contains(&entry.chance) || !entry.chance.is_finite() {
                report.errors.push(format!(
                    "{path}:entries[{index}].chance: must be finite and within 0..=1"
                ));
            }
        }
    }
}
