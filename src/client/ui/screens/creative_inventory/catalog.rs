//! 构建创造物品栏的分类索引，并计算当前筛选结果。

use std::collections::HashSet;

use bevy::prelude::*;

use crate::client::ui::components::CreativeCategoryPanel;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::category_theme::CategoryTheme;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::tab::spawn_category_tab;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::definition::ItemCategory;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::engine::localization::Localization;
use crate::game::inventory::container::creative::CreativeCategory;
use crate::game::inventory::state::LocalInventoryMut;
use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagId;
/// 构造创造模式分类数据，并按截图风格固定分类顺序。
/// 分类构建依赖多个独立内容注册表，缺少任一可选资源时分别降级。
#[allow(clippy::too_many_arguments)]
pub fn build_creative_categories_system(
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    block_registry: Option<Res<BlockRegistry>>,
    mut state: LocalInventoryMut,
    category_panel: Query<Entity, With<CreativeCategoryPanel>>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    cat_theme: Res<CategoryTheme>,
    localization: Res<Localization>,
    mut commands: Commands,
    item_registry: Option<Res<ItemRegistry>>,
) {
    let Some(tag_reg) = tag_registry else { return };
    let Some(block_reg) = block_registry else {
        return;
    };
    if !state.creative.categories.is_empty() {
        return;
    }

    // 这些标签来自数据层；显示顺序固定，避免 HashMap 顺序导致 UI 抖动。
    let mut categories = vec![
        CreativeCategory::virtual_category("creative.category.all", "All", "■"),
        category_from_tag(
            "solid",
            "creative.category.solid",
            "Solid",
            "▣",
            &tag_reg,
            &block_reg,
        ),
        category_from_tag(
            "tree_plantable",
            "creative.category.crop",
            "Crop",
            "♧",
            &tag_reg,
            &block_reg,
        ),
        category_from_tag(
            "natural",
            "creative.category.natural",
            "Natural",
            "♣",
            &tag_reg,
            &block_reg,
        ),
    ];

    let mut tools = CreativeCategory::virtual_category("creative.category.tools", "Tools", "⚒");
    if let Some(item_reg) = item_registry.as_ref() {
        let mut tool_items = item_reg.items_by_category(&ItemCategory::Tool).to_vec();
        // 注册顺序由内容编译决定，不一定稳定；按 identifier 排序保持槽位一致。
        tool_items.sort_by(|a, b| a.identifier().path().cmp(b.identifier().path()));
        tools.items = tool_items;
    }
    categories.push(tools);

    // 预留与参考图一致的分类入口，后续有对应数据时只需要填充 items。
    categories.push(CreativeCategory::virtual_category(
        "creative.category.decor",
        "Decor",
        "▤",
    ));
    categories.push(CreativeCategory::virtual_category(
        "creative.category.redstone",
        "Redstone",
        "◆",
    ));
    categories.push(CreativeCategory::virtual_category(
        "creative.category.transport",
        "Transport",
        "≡",
    ));
    categories.push(CreativeCategory::virtual_category(
        "creative.category.misc",
        "Misc",
        "◒",
    ));
    categories.push(CreativeCategory::virtual_category(
        "creative.category.favorites",
        "Favorites",
        "☆",
    ));

    // 追加未显式列出的数据标签，保证新增标签不会被 UI 吞掉。
    let known = [
        "century_journey:solid",
        "century_journey:tree_plantable",
        "century_journey:natural",
    ];
    // 标签来自 HashMap 迭代，顺序不稳定；排序保证分类标签顺序可复现。
    let mut extra_tags: Vec<&TagId> = tag_reg
        .all_tags()
        .filter(|tag| !known.contains(&tag.to_full().as_str()))
        .collect();
    extra_tags.sort_by_key(|tag| tag.to_full());
    for tag in extra_tags {
        let tag_full = tag.to_full();
        categories.push(CreativeCategory::from_tag(
            tag.clone(),
            category_label_key(tag),
            category_label_fallback(tag),
            cat_theme.icon(&tag_full),
            items_for_tag(tag, &tag_reg, &block_reg),
        ));
    }

    state.creative.categories = categories;

    let Ok(panel_entity) = category_panel.single() else {
        return;
    };
    commands.entity(panel_entity).with_children(|panel| {
        for (idx, cat) in state.creative.categories.iter().enumerate() {
            spawn_category_tab(
                panel,
                cat,
                idx,
                idx == state.creative.selected_tab,
                &ui_font,
                &theme,
                &localization,
            );
        }
    });
}

/// 从指定方块标签生成本地化键形式的创造模式分类。
fn category_from_tag(
    path: &str,
    label_key: &str,
    label_fallback: &str,
    icon: &str,
    tag_registry: &RuntimeTagRegistry,
    block_registry: &BlockRegistry,
) -> CreativeCategory {
    let tag_id = TagId::new("century_journey", path);
    CreativeCategory::from_tag(
        tag_id.clone(),
        label_key.to_string(),
        label_fallback.to_string(),
        icon.to_string(),
        items_for_tag(&tag_id, tag_registry, block_registry),
    )
}

/// 由标签派生分类名的本地化键。
///
/// 默认命名空间只取路径；其他命名空间（如 `mineable:axe`）保留
/// `命名空间-路径` 前缀，避免不同命名空间的同名标签键冲突。
/// 路径中的下划线归一化为连字符，与语言文件键名惯例一致。
fn category_label_key(tag: &TagId) -> String {
    let namespace = tag.namespace();
    let path = tag.path().replace('_', "-");
    if namespace == "century_journey" {
        format!("creative.category.{path}")
    } else {
        format!("creative.category.{namespace}-{path}")
    }
}

/// 由标签派生键缺失时的兜底名。
/// 默认命名空间取路径首字母大写；其他命名空间保留 `命名空间/路径` 原样。
fn category_label_fallback(tag: &TagId) -> String {
    if tag.namespace() != "century_journey" {
        return tag.to_full();
    }
    let path = tag.path();
    let mut chars = path.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// 将方块标签里的运行时方块 ID 转换成物品 ID。
///
/// `BlockRegistry::identifier_to_id` 与 `ItemRegistry::entries` 都是 HashMap，
/// 迭代顺序不稳定会直接反映到槽位上：同一物品每次启动在不同槽位、点击拾取
/// 也指向不同 identifier。返回前按 identifier.path 排序，保证槽位 ↔ 物品
/// 的映射稳定。
fn items_for_tag(
    tag: &TagId,
    tag_registry: &RuntimeTagRegistry,
    block_registry: &BlockRegistry,
) -> Vec<ItemId> {
    tag_registry
        .get_ids(tag)
        .map(|ids| {
            let mut items: Vec<ItemId> = ids
                .iter()
                .filter_map(|&id| block_registry.get_identifier_by_id(id))
                .map(|ident| ItemId::new(ident.clone()))
                .collect();
            items.sort_by(|a, b| a.identifier().path().cmp(b.identifier().path()));
            items
        })
        .unwrap_or_default()
}

/// 搜索过滤更新。
pub fn update_creative_filter_system(
    block_registry: Option<Res<BlockRegistry>>,
    item_registry: Option<Res<ItemRegistry>>,
    mut state: LocalInventoryMut,
) {
    let Some(reg) = block_registry else { return };
    if state.creative.categories.is_empty() {
        return;
    }

    let tab = state.creative.selected_tab;
    let search = state.creative.search_text.clone();

    let mut new_items = if tab == 0 {
        // “全部”分类：方块 + 物品，去重后统一展示。
        let mut seen = HashSet::new();
        let mut all: Vec<ItemId> = Vec::new();

        for id in reg.identifiers() {
            if id == "century_journey:air" {
                continue;
            }
            let item_id = ItemId::new(id.clone());
            if seen.insert(item_id.clone()) {
                all.push(item_id);
            }
        }

        if let Some(item_reg) = item_registry.as_ref() {
            for def in item_reg.all_items() {
                let item_id = ItemId::new(def.identifier.clone());
                if seen.insert(item_id.clone()) {
                    all.push(item_id);
                }
            }
        }
        // 注册表底层 HashMap 迭代顺序不稳定，会导致同一物品每次启动在不同槽位；
        // 按 identifier 路径排序，让槽位 ↔ 物品的对应关系稳定可预期。
        all.sort_by(|a, b| a.identifier().path().cmp(b.identifier().path()));
        all
    } else if let Some(cat) = state.creative.categories.get(tab) {
        if cat.label_key == "creative.category.favorites" {
            state.creative.favorites.clone()
        } else {
            cat.items.clone()
        }
    } else {
        Vec::new()
    };

    if !search.is_empty() {
        let keyword = search.to_lowercase();
        new_items.retain(|item| item.to_string().to_lowercase().contains(&keyword));
    }

    if state.creative.visible_items != new_items {
        state.creative.visible_items = new_items;
    }
}
