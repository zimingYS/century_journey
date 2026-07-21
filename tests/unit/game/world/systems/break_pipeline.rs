use super::*;
use crate::content::item::definition::tool::{ToolTier, ToolType};

#[test]
fn requested_fix_block_drop_starts_inside_broken_voxel() {
    let block = IVec3::new(3, 12, -4);
    let position = block_drop_spawn_position(block, 0);

    assert!((block.x as f32..block.x as f32 + 1.0).contains(&position.x));
    assert_eq!(position.y, block.y as f32 + 0.5);
    assert!((block.z as f32..block.z as f32 + 1.0).contains(&position.z));
}

#[test]
fn survival_loot_requires_matching_tool_and_sufficient_tier() {
    let block = BlockProperty {
        required_tool: Some(ToolType::Pickaxe),
        harvest_level: 1,
        ..default()
    };
    let survival = PlayerGameMode::default();
    let wood_pickaxe = ToolData::new(ToolType::Pickaxe, ToolTier::Wood, 60, 1.5);
    let stone_axe = ToolData::new(ToolType::Axe, ToolTier::Stone, 132, 2.0);
    let stone_pickaxe = ToolData::new(ToolType::Pickaxe, ToolTier::Stone, 132, 2.0);

    assert!(!should_drop_block_loot(&survival, &block, None));
    assert!(!should_drop_block_loot(
        &survival,
        &block,
        Some(&wood_pickaxe)
    ));
    assert!(!should_drop_block_loot(&survival, &block, Some(&stone_axe)));
    assert!(should_drop_block_loot(
        &survival,
        &block,
        Some(&stone_pickaxe)
    ));

    let wood = BlockProperty {
        required_tool: None,
        effective_tool: Some(ToolType::Axe),
        ..default()
    };
    assert!(should_drop_block_loot(&survival, &wood, None));
}
