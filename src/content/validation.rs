//! 内容定义的全局编译入口和校验报告。
//!
//! 本模块先完成全部定义的加载与交叉引用检查，再生成按稳定标识符排序的运行时数据，
//! 防止文件系统遍历顺序改变动态 ID 和世界结果。

use crate::content::biome::BiomeDefinition;
use crate::content::block::definition::BlockProperty;
use crate::content::block::model::BlockModel;
use crate::content::format::versioned_json_dir_results;
use crate::content::item::definition::ItemDefinition;
use crate::content::item::texture::icon::IconDefinition;
use crate::content::loot::table::LootTable;
use crate::content::recipe::definition::Ingredient;
use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::content::tag::definition::TagAction;
use crate::engine::asset::{AssetFiles, AssetResolver};
use crate::shared::held_item::HeldRenderDefinition;
use crate::shared::identifier::Identifier;
use crate::shared::tag::identifier::TagId;
use bevy::prelude::Resource;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
/// 一次内容检查的统计结果和可定位错误列表。
pub struct ContentCheckReport {
    pub checked_files: usize,
    pub errors: Vec<String>,
}

impl ContentCheckReport {
    /// 仅当没有发现错误时，内容才允许进入运行时注册表。
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// 全局校验完成后唯一允许进入运行时注册表的数据集合。
///
/// 每个列表都已按稳定标识符排序，运行时动态 ID 不再取决于文件系统遍历顺序。
#[derive(Debug, Clone, Default)]
pub struct CompiledContent {
    pub blocks: Vec<BlockProperty>,
    pub items: Vec<ItemDefinition>,
    pub biomes: Vec<BiomeDefinition>,
    pub recipes: Vec<(Identifier, RecipeDefinition)>,
    pub block_loot: Vec<(Identifier, LootTable)>,
    pub tags: Vec<(TagId, TagAction)>,
}

#[derive(Resource, Debug, Clone, Default)]
/// 启动阶段共享的内容编译结果资源。
pub struct ContentCompilation {
    pub report: ContentCheckReport,
    pub content: CompiledContent,
}

impl ContentCompilation {
    /// 返回本次编译是否可以安全构建运行时注册表。
    pub fn is_valid(&self) -> bool {
        self.report.is_valid()
    }

    /// 生成适合启动错误界面展示的摘要，并限制显示条目数量。
    pub fn error_summary(&self, limit: usize) -> String {
        let shown = self
            .report
            .errors
            .iter()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        let remaining = self.report.errors.len().saturating_sub(limit);
        if remaining == 0 {
            shown
        } else {
            format!("{shown}\n... 另有 {remaining} 个错误")
        }
    }
}

/// 执行完整内容检查，但不向调用者暴露编译后的定义集合。
pub fn check_content(resolver: &AssetResolver) -> ContentCheckReport {
    compile_content(resolver).report
}

/// 加载、交叉校验并稳定排序全部数据驱动内容。
pub fn compile_content(resolver: &AssetResolver) -> ContentCompilation {
    let files = AssetFiles::new(resolver);
    let mut report = ContentCheckReport::default();

    let mut blocks = load::<BlockProperty>(&files, "definitions/blocks", &mut report);
    let mut items = load::<ItemDefinition>(&files, "definitions/items", &mut report);
    let mut biomes = load::<BiomeDefinition>(&files, "definitions/biomes", &mut report);
    let mut recipes = load::<RecipeDefinition>(&files, "definitions/recipes", &mut report);
    let mut loot = load::<LootTable>(&files, "definitions/loot/blocks", &mut report);
    let mut tags = load::<TagAction>(&files, "definitions/tags", &mut report);

    let block_ids = unique_ids(
        blocks
            .iter()
            .map(|(path, block)| (path, block.identifier.to_string())),
        "block",
        &mut report,
    );
    let explicit_item_ids = unique_ids(
        items
            .iter()
            .map(|(path, item)| (path, item.identifier.to_string())),
        "item",
        &mut report,
    );
    let mut item_ids = explicit_item_ids.clone();
    for block_id in &block_ids {
        if explicit_item_ids.contains(block_id) {
            report.errors.push(format!(
                "definitions/items:identifier: explicit item {block_id} conflicts with the generated block item"
            ));
        }
        item_ids.insert(block_id.clone());
    }

    if !block_ids.contains("century_journey:air") {
        report
            .errors
            .push("definitions/blocks: missing century_journey:air".into());
    }

    validate_blocks(resolver, &blocks, &block_ids, &mut report);
    validate_items(resolver, &items, &block_ids, &mut report);
    validate_biomes(&biomes, &block_ids, &mut report);
    let mut item_tag_ids = tags
        .iter()
        .filter_map(|(path, _)| tag_identity(path))
        .filter_map(|identity| identity.strip_prefix("item:").map(ToOwned::to_owned))
        .collect::<HashSet<_>>();
    for (_, item) in &items {
        item_tag_ids.extend(item.tags.iter().map(|tag| inline_tag_id(tag).to_full()));
    }
    validate_recipes(&recipes, &item_ids, &item_tag_ids, &mut report);
    validate_loot(&loot, &block_ids, &item_ids, &mut report);
    validate_tags(&tags, &block_ids, &item_ids, &biomes, &mut report);
    validate_textures(resolver, &files, &mut report);

    blocks.sort_by(|left, right| {
        left.1
            .identifier
            .cmp(&right.1.identifier)
            .then_with(|| left.0.cmp(&right.0))
    });
    items.sort_by(|left, right| {
        left.1
            .identifier
            .cmp(&right.1.identifier)
            .then_with(|| left.0.cmp(&right.0))
    });
    biomes.sort_by(|left, right| {
        left.1
            .generation_order
            .cmp(&right.1.generation_order)
            .then_with(|| left.1.identifier.cmp(&right.1.identifier))
            .then_with(|| left.0.cmp(&right.0))
    });
    recipes.sort_by(|left, right| left.0.cmp(&right.0));
    loot.sort_by(|left, right| left.0.cmp(&right.0));
    tags.sort_by(|left, right| left.0.cmp(&right.0));

    let mut recipe_ids = HashSet::new();
    let recipes = recipes
        .into_iter()
        .filter_map(|(path, recipe)| {
            let Some(identifier) = recipe_id(&path) else {
                report
                    .errors
                    .push(format!("{path}:path: invalid recipe definition path"));
                return None;
            };
            if !recipe_ids.insert(identifier.clone()) {
                report.errors.push(format!(
                    "{path}:path: duplicate recipe identifier {identifier}"
                ));
                return None;
            }
            Some((identifier, recipe))
        })
        .collect();
    let mut loot_ids = HashSet::new();
    let block_loot = loot
        .into_iter()
        .filter_map(|(path, table)| {
            let Some(identifier) = block_loot_id(&path) else {
                report
                    .errors
                    .push(format!("{path}:path: invalid block loot definition path"));
                return None;
            };
            if !loot_ids.insert(identifier.clone()) {
                report.errors.push(format!(
                    "{path}:path: duplicate block loot identifier {identifier}"
                ));
                return None;
            }
            Some((identifier, table))
        })
        .collect();
    let tags = tags
        .into_iter()
        .filter_map(|(path, action)| {
            tag_runtime_id(&path).map(|id| (id, action)).or_else(|| {
                report
                    .errors
                    .push(format!("{path}:path: invalid tag definition path"));
                None
            })
        })
        .collect();

    ContentCompilation {
        report,
        content: CompiledContent {
            blocks: blocks.into_iter().map(|(_, value)| value).collect(),
            items: items.into_iter().map(|(_, value)| value).collect(),
            biomes: biomes.into_iter().map(|(_, value)| value).collect(),
            recipes,
            block_loot,
            tags,
        },
    }
}

mod validation_checks;
use validation_checks::*;
#[cfg(test)]
#[path = "../../tests/unit/content/validation.rs"]
mod tests;
