use super::*;
use crate::content::item::definition::presentation::{AnimationConfig, HeldRenderDefinition};
use crate::content::item::definition::tool::{ToolData, ToolTier, ToolType};
use crate::engine::localization::{LanguageId, LanguageInfo, Localization};
use crate::shared::identifier::Identifier;
use std::collections::BTreeMap;

/// 构造包含提示框文案键的中文本地化资源。
fn tooltip_localization() -> Localization {
    let entries = [
        ("tooltip.category", "类别  {name}"),
        ("tooltip.categories.tool", "工具"),
        ("tooltip.max-stack", "最大堆叠  {count}"),
        ("tooltip.tool-type", "工具类型  {type}"),
        ("tooltip.tool-types.pickaxe", "镐"),
        ("tooltip.tier", "工具等级  {tier}"),
        ("tooltip.tier.iron", "铁质"),
        ("tooltip.efficiency", "效率  {value}x"),
        ("tooltip.durability", "耐久上限  {count}"),
        ("tooltip.tags", "标签  {tags}"),
        ("item.test.pickaxe", "测试镐"),
    ];
    let mut zh_table = BTreeMap::new();
    for (key, text) in entries {
        zh_table.insert(key.to_string(), text.to_string());
    }
    let mut tables = BTreeMap::new();
    tables.insert(LanguageId::new("zh-CN"), zh_table);
    Localization::new(
        vec![LanguageInfo {
            id: LanguageId::new("zh-CN"),
            native_name: "简体中文".to_string(),
        }],
        tables,
    )
}

#[test]
fn tool_tooltip_includes_category_attributes_and_durability() {
    let definition = ItemDefinition {
        identifier: Identifier::parse("test:pickaxe").unwrap(),
        display_name: "Test pickaxe".into(),
        category: ItemCategory::Tool,
        max_stack: 1,
        tags: vec!["tools".into()],
        icon: default(),
        model: None,
        placeable_block: None,
        tool: Some(ToolData::new(ToolType::Pickaxe, ToolTier::Iron, 250, 6.0)),
        food: None,
        drink: None,
        held_renderer: HeldRenderDefinition::default(),
        animations: AnimationConfig::default(),
    };
    let (title, body) = tooltip_text(&definition, &tooltip_localization());
    assert_eq!(title, "测试镐");
    assert!(body.contains("类别  工具"));
    assert!(body.contains("效率  6.0x"));
    assert!(body.contains("耐久上限  250"));
}

#[test]
fn missing_name_key_falls_back_to_display_name() {
    let definition = ItemDefinition {
        identifier: Identifier::parse("test:unknown_thing").unwrap(),
        display_name: "原始名称".into(),
        category: ItemCategory::Material,
        max_stack: 16,
        tags: Vec::new(),
        icon: default(),
        model: None,
        placeable_block: None,
        tool: None,
        food: None,
        drink: None,
        held_renderer: HeldRenderDefinition::default(),
        animations: AnimationConfig::default(),
    };
    let (title, _) = tooltip_text(&definition, &tooltip_localization());
    assert_eq!(title, "原始名称");
}
