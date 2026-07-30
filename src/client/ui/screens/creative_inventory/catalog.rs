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
        CreativeCategory::virtual_category("全部", "■"),
        category_from_tag("solid", "固体", "▣", &tag_reg, &block_reg),
        category_from_tag("tree_plantable", "作物", "♧", &tag_reg, &block_reg),
        category_from_tag("natural", "自然", "♣", &tag_reg, &block_reg),
    ];

    let mut tools = CreativeCategory::virtual_category("工具", "⚒");
    if let Some(item_reg) = item_registry.as_ref() {
        tools.items = item_reg.items_by_category(&ItemCategory::Tool).to_vec();
    }
    categories.push(tools);

    // 预留与参考图一致的分类入口，后续有对应数据时只需要填充 items。
    categories.push(CreativeCategory::virtual_category("装饰", "▤"));
    categories.push(CreativeCategory::virtual_category("红石", "◆"));
    categories.push(CreativeCategory::virtual_category("运输", "≡"));
    categories.push(CreativeCategory::virtual_category("杂项", "◒"));
    categories.push(CreativeCategory::virtual_category("收藏", "☆"));

    // 追加未显式列出的数据标签，保证新增标签不会被 UI 吞掉。
    let known = [
        "century_journey:solid",
        "century_journey:tree_plantable",
        "century_journey:natural",
    ];
    for tag in tag_reg.all_tags() {
        let tag_full = tag.to_full();
        if known.contains(&tag_full.as_str()) {
            continue;
        }
        categories.push(CreativeCategory::from_tag(
            tag.clone(),
            cat_theme.display_name(&tag_full),
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
                &cat.display_name,
                &cat.icon,
                idx,
                idx == state.creative.selected_tab,
                &ui_font,
                &theme,
            );
        }
    });
}

/// 从指定方块标签生成创造模式分类。
fn category_from_tag(
    path: &str,
    display_name: &str,
    icon: &str,
    tag_registry: &RuntimeTagRegistry,
    block_registry: &BlockRegistry,
) -> CreativeCategory {
    let tag_id = TagId::new("century_journey", path);
    CreativeCategory::from_tag(
        tag_id.clone(),
        display_name.to_string(),
        icon.to_string(),
        items_for_tag(&tag_id, tag_registry, block_registry),
    )
}

/// 将方块标签里的运行时方块 ID 转换成物品 ID。
fn items_for_tag(
    tag: &TagId,
    tag_registry: &RuntimeTagRegistry,
    block_registry: &BlockRegistry,
) -> Vec<ItemId> {
    tag_registry
        .get_ids(tag)
        .map(|ids| {
            ids.iter()
                .filter_map(|&id| block_registry.get_identifier_by_id(id))
                .map(|ident| ItemId::new(ident.clone()))
                .collect()
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
        all
    } else if let Some(cat) = state.creative.categories.get(tab) {
        if cat.tag_id.is_none() && cat.display_name == "收藏" {
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
