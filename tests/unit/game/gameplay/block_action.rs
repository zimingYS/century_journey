use super::*;
use crate::content::item::definition::tool::{ToolTier, ToolType};
use crate::content::loot::block_registry::BlockLootRegistry;
use crate::content::loot::table::LootTable;
use crate::shared::random::DeterministicRng;

fn pickaxe(tier: ToolTier, efficiency: f32) -> ToolData {
    ToolData::new(ToolType::Pickaxe, tier, 100, efficiency)
}

fn axe(tier: ToolTier, efficiency: f32) -> ToolData {
    ToolData::new(ToolType::Axe, tier, 100, efficiency)
}

fn shovel(tier: ToolTier, efficiency: f32) -> ToolData {
    ToolData::new(ToolType::Shovel, tier, 100, efficiency)
}

#[test]
fn harvest_requires_matching_tool_and_sufficient_tier() {
    let block = BlockProperty {
        required_tool: Some(ToolType::Pickaxe),
        harvest_level: 1,
        ..default()
    };

    let wood_pickaxe = pickaxe(ToolTier::Wood, 1.0);
    let stone_axe = axe(ToolTier::Stone, 1.0);
    let stone_pickaxe = pickaxe(ToolTier::Stone, 1.0);

    assert!(!can_harvest_block(&block, None));
    assert!(!can_harvest_block(&block, Some(&wood_pickaxe)));
    assert!(!can_harvest_block(&block, Some(&stone_axe)));
    assert!(can_harvest_block(&block, Some(&stone_pickaxe)));
}

#[test]
fn wrong_tool_or_low_tier_can_break_block_with_speed_penalty() {
    let block = BlockProperty {
        hardness: 3.0,
        required_tool: Some(ToolType::Pickaxe),
        harvest_level: 1,
        ..default()
    };
    let gamemode = PlayerGameMode::default();
    let wood_pickaxe = pickaxe(ToolTier::Wood, 1.5);
    let stone_axe = axe(ToolTier::Stone, 2.0);
    let stone_pickaxe = pickaxe(ToolTier::Stone, 2.0);

    assert_eq!(block_break_seconds(&block, &gamemode, None), Some(15.0));
    assert_eq!(
        block_break_seconds(&block, &gamemode, Some(&wood_pickaxe)),
        Some(15.0)
    );
    assert_eq!(
        block_break_seconds(&block, &gamemode, Some(&stone_axe)),
        Some(15.0)
    );
    assert_eq!(
        block_break_seconds(&block, &gamemode, Some(&stone_pickaxe)),
        Some(1.5)
    );
}

#[test]
fn matching_tool_efficiency_reduces_break_seconds() {
    let block = BlockProperty {
        hardness: 3.0,
        required_tool: Some(ToolType::Pickaxe),
        ..default()
    };

    let slow = pickaxe(ToolTier::Wood, 1.0);
    let fast = pickaxe(ToolTier::Iron, 3.0);
    let gamemode = PlayerGameMode::default();

    let slow_seconds = block_break_seconds(&block, &gamemode, Some(&slow)).unwrap();
    let fast_seconds = block_break_seconds(&block, &gamemode, Some(&fast)).unwrap();

    assert!(fast_seconds < slow_seconds);
    assert_eq!(slow_seconds, 3.0);
    assert_eq!(fast_seconds, 1.0);
}

#[test]
fn optional_effective_tool_accelerates_wood_without_restricting_harvest() {
    let block = BlockProperty {
        hardness: 1.0,
        required_tool: None,
        effective_tool: Some(ToolType::Axe),
        ..default()
    };
    let gamemode = PlayerGameMode::default();
    let wooden_axe = axe(ToolTier::Wood, 1.2);
    let stone_axe = axe(ToolTier::Stone, 1.5);
    let iron_axe = axe(ToolTier::Iron, 2.0);
    let iron_pickaxe = pickaxe(ToolTier::Iron, 3.0);

    assert!(can_harvest_block(&block, None));
    let hand_seconds = block_break_seconds(&block, &gamemode, None).unwrap();
    let wooden_axe_seconds = block_break_seconds(&block, &gamemode, Some(&wooden_axe)).unwrap();
    let stone_axe_seconds = block_break_seconds(&block, &gamemode, Some(&stone_axe)).unwrap();
    let iron_axe_seconds = block_break_seconds(&block, &gamemode, Some(&iron_axe)).unwrap();

    assert_eq!(hand_seconds, 1.0);
    assert_eq!(
        block_break_seconds(&block, &gamemode, Some(&iron_pickaxe)),
        Some(hand_seconds)
    );
    assert!(wooden_axe_seconds < hand_seconds);
    assert!(stone_axe_seconds < wooden_axe_seconds);
    assert!(iron_axe_seconds < stone_axe_seconds);
}

#[test]
fn optional_shovel_accelerates_dirt_without_restricting_harvest() {
    let block = BlockProperty {
        hardness: 0.5,
        required_tool: None,
        effective_tool: Some(ToolType::Shovel),
        ..default()
    };
    let gamemode = PlayerGameMode::default();
    let wooden_shovel = shovel(ToolTier::Wood, 1.5);

    assert!(can_harvest_block(&block, None));
    assert_eq!(block_break_seconds(&block, &gamemode, None), Some(0.5));
    assert!(block_break_seconds(&block, &gamemode, Some(&wooden_shovel)).unwrap() < 0.5);
}

#[test]
fn break_state_resets_when_target_or_tool_changes() {
    let mut state = BlockBreakState::default();
    let wood_pickaxe = ItemId::item("century_journey:wooden_pickaxe");
    let stone_pickaxe = ItemId::item("century_journey:stone_pickaxe");

    assert_eq!(
        state.tick(IVec3::new(1, 2, 3), 5, &wood_pickaxe, 0.25),
        0.25
    );
    assert_eq!(state.tick(IVec3::new(1, 2, 3), 5, &wood_pickaxe, 0.25), 0.5);
    assert_eq!(
        state.tick(IVec3::new(1, 2, 4), 5, &wood_pickaxe, 0.25),
        0.25
    );
    assert_eq!(
        state.tick(IVec3::new(1, 2, 4), 5, &stone_pickaxe, 0.25),
        0.25
    );
}

#[test]
fn stage_seven_mining_rule_produces_registered_survival_drop() {
    let block = BlockProperty {
        hardness: 0.5,
        required_tool: None,
        ..default()
    };
    let gamemode = PlayerGameMode::default();
    assert_eq!(block_break_seconds(&block, &gamemode, None), Some(0.5));

    let dirt = ItemId::block("century_journey:dirt");
    let mut loot = BlockLootRegistry::default();
    loot.register(7, LootTable::single(dirt.clone(), 1));

    let mut rng = DeterministicRng::new(7);
    assert_eq!(loot.roll(7, &mut rng), vec![(dirt, 1)]);
}
