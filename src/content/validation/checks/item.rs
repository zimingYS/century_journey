//! 物品定义、工具属性和表现资源引用校验。

use super::super::{ContentCheckReport, paths::content_file_exists};
use crate::content::item::definition::ItemDefinition;
use crate::content::item::definition::presentation::HeldRenderDefinition;
use crate::content::item::texture::icon::IconDefinition;
use crate::engine::asset::AssetResolver;
use crate::shared::identifier::Identifier;
use std::collections::{BTreeSet, HashSet};

/// 校验物品标识符、工具属性与表现资源引用。
pub(in crate::content::validation) fn validate_items(
    resolver: &AssetResolver,
    items: &[(String, ItemDefinition)],
    block_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, item) in items {
        if item.max_stack == 0 {
            report
                .errors
                .push(format!("{path}:max_stack: must be positive"));
        }
        if let Some(block) = &item.placeable_block
            && !block_ids.contains(&block.to_string())
        {
            report
                .errors
                .push(format!("{path}:placeable_block: unknown block {block}"));
        }
        if let Some(tool) = &item.tool {
            if tool.max_durability == 0 {
                report
                    .errors
                    .push(format!("{path}:tool.max_durability: must be positive"));
            }
            if !tool.efficiency.is_finite() || tool.efficiency <= 0.0 {
                report.errors.push(format!(
                    "{path}:tool.efficiency: must be finite and positive"
                ));
            }
        }
        if let Some(food) = &item.food
            && (!food.hunger.is_finite()
                || !food.saturation.is_finite()
                || food.hunger < 0.0
                || food.saturation < 0.0)
        {
            report.errors.push(format!(
                "{path}:food: hunger and saturation must be finite and non-negative"
            ));
        }
        match &item.held_renderer {
            HeldRenderDefinition::FlatItem { thickness }
                if !thickness.is_finite() || *thickness <= 0.0 =>
            {
                report.errors.push(format!(
                    "{path}:held_renderer.thickness: must be finite and positive"
                ));
            }
            HeldRenderDefinition::Model { path: model_path } if model_path.trim().is_empty() => {
                report
                    .errors
                    .push(format!("{path}:held_renderer.path: must not be empty"));
            }
            _ => {}
        }
        match &item.icon {
            IconDefinition::Block(block) if !block_ids.contains(&block.to_string()) => {
                report
                    .errors
                    .push(format!("{path}:icon.value: unknown block {block}"));
            }
            IconDefinition::Texture(identifier) => match Identifier::parse(identifier) {
                Ok(identifier) => {
                    let texture = format!("textures/items/{}.png", identifier.path());
                    if !content_file_exists(resolver, &texture) {
                        report
                            .errors
                            .push(format!("{path}:icon.value: missing item texture {texture}"));
                    }
                }
                Err(error) => report.errors.push(format!(
                    "{path}:icon.value: invalid texture identifier: {error}"
                )),
            },
            IconDefinition::Block(_) => {}
        }
    }
}

/// 校验物品名称的本地化键在回退语言中存在。
///
/// 界面按 `item.<命名空间>.<路径>` 查询物品名；键缺失时会退回 JSON 的
/// `display_name`，导致中英文界面混排，因此在离线检查阶段拦截。
/// 方块会桥接为同名物品参与校验，`air` 不注册为物品故跳过。
pub(in crate::content::validation) fn validate_item_name_keys(
    items: &[(String, ItemDefinition)],
    block_ids: &HashSet<String>,
    fallback_keys: &BTreeSet<String>,
    report: &mut ContentCheckReport,
) {
    let item_keys = items
        .iter()
        .map(|(path, item)| (path.clone(), item.name_key()));
    let block_keys = block_ids
        .iter()
        .filter(|id| id.as_str() != "century_journey:air")
        .map(|id| {
            let (namespace, path) = id
                .split_once(':')
                .unwrap_or(("century_journey", id.as_str()));
            (
                format!("definitions/blocks/{namespace}/{path}.json"),
                format!("item.{namespace}.{path}"),
            )
        });
    for (path, key) in item_keys.chain(block_keys) {
        if !fallback_keys.contains(&key) {
            report
                .errors
                .push(format!("{path}:name: missing locale key {key}"));
        }
    }
}
