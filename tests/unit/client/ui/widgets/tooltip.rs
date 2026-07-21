use super::*;
use crate::content::item::definition::tool::{ToolData, ToolTier, ToolType};
use crate::shared::held_item::{AnimationConfig, HeldRenderDefinition};
use crate::shared::identifier::Identifier;

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
        held_renderer: HeldRenderDefinition::default(),
        animations: AnimationConfig::default(),
    };
    let (_, body) = tooltip_text(&definition);
    assert!(body.contains("类别  工具"));
    assert!(body.contains("效率  6.0x"));
    assert!(body.contains("耐久上限  250"));
}
