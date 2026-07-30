//! 标签路径、成员引用和同领域标签引用校验。

use super::super::{ContentCheckReport, paths::tag_identity};
use crate::content::biome::BiomeDefinition;
use crate::content::tag::definition::TagAction;
use std::collections::HashSet;

/// 校验标签路径、成员引用与同领域标签引用。
pub(in crate::content::validation) fn validate_tags(
    tags: &[(String, TagAction)],
    block_ids: &HashSet<String>,
    item_ids: &HashSet<String>,
    biomes: &[(String, BiomeDefinition)],
    report: &mut ContentCheckReport,
) {
    let biome_ids: HashSet<_> = biomes
        .iter()
        .map(|(_, biome)| biome.identifier.to_string())
        .collect();
    let tag_keys: HashSet<_> = tags
        .iter()
        .filter_map(|(path, _)| tag_identity(path))
        .collect();
    for (path, action) in tags {
        let Some((kind, _)) = tag_identity(path).and_then(|value| {
            value
                .split_once(':')
                .map(|(kind, rest)| (kind.to_string(), rest.to_string()))
        }) else {
            report.errors.push(format!("{path}:path: invalid tag path"));
            continue;
        };
        let known = match kind.as_str() {
            "block" => block_ids,
            "item" => item_ids,
            "biome" => &biome_ids,
            _ => {
                report
                    .errors
                    .push(format!("{path}:path: unsupported tag kind {kind}"));
                continue;
            }
        };
        for value in tag_values(action) {
            if let Some(reference) = value.strip_prefix('#') {
                if !tag_keys.contains(&format!("{kind}:{reference}")) {
                    report
                        .errors
                        .push(format!("{path}:values: unresolved tag reference {value}"));
                }
            } else if !known.contains(value) {
                report
                    .errors
                    .push(format!("{path}:values: unknown {kind} member {value}"));
            }
        }
    }
}

/// 返回标签动作携带的成员列表，供统一引用检查使用。
fn tag_values(action: &TagAction) -> &[String] {
    match action {
        TagAction::Append { append } => append,
        TagAction::Remove { remove } => remove,
        TagAction::Replace { replace } => replace,
        TagAction::Values { values, .. } => values,
    }
}
