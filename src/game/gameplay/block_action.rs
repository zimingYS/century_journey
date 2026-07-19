use crate::content::block::definition::BlockProperty;
use crate::content::item::definition::tool::ToolData;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagId;
use bevy::prelude::*;

const BLOCK_TAG_NAMESPACE: &str = "century_journey";
const UNBREAKABLE_TAG: &str = "unbreakable";
const REPLACEABLE_TAG: &str = "overworld_replaceable";
const BASE_BREAK_SECONDS_PER_HARDNESS: f32 = 1.0;
const MIN_SURVIVAL_BREAK_SECONDS: f32 = 0.08;
const MIN_TOOL_EFFICIENCY: f32 = 0.1;
const INCORRECT_TOOL_BREAK_TIME_MULTIPLIER: f32 = 5.0;

#[derive(Resource, Debug, Clone)]
pub struct BlockBreakProgress {
    pub visible: bool,
    pub world_pos: IVec3,
    pub block_id: u16,
    pub progress: f32,
}

impl Default for BlockBreakProgress {
    fn default() -> Self {
        Self {
            visible: false,
            world_pos: IVec3::ZERO,
            block_id: 0,
            progress: 0.0,
        }
    }
}

impl BlockBreakProgress {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn set(&mut self, world_pos: IVec3, block_id: u16, progress: f32) {
        self.visible = true;
        self.world_pos = world_pos;
        self.block_id = block_id;
        self.progress = progress.clamp(0.0, 1.0);
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockBreakState {
    target: Option<BlockBreakTarget>,
    elapsed_seconds: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockBreakTarget {
    world_pos: IVec3,
    block_id: u16,
    tool_item: ItemId,
}

impl BlockBreakState {
    pub fn clear(&mut self) {
        self.target = None;
        self.elapsed_seconds = 0.0;
    }

    pub fn tick(&mut self, world_pos: IVec3, block_id: u16, tool_item: &ItemId, delta: f32) -> f32 {
        let next_target = BlockBreakTarget {
            world_pos,
            block_id,
            tool_item: tool_item.clone(),
        };

        if self.target.as_ref() != Some(&next_target) {
            self.target = Some(next_target);
            self.elapsed_seconds = 0.0;
        }

        self.elapsed_seconds += delta.max(0.0);
        self.elapsed_seconds
    }
}

pub fn can_place_block(
    block_id: u16,
    gamemode: &PlayerGameMode,
    active_stack: Option<&ItemStack>,
) -> bool {
    if block_id == 0 {
        return false;
    }
    if gamemode.is_creative() {
        return true;
    }
    active_stack.is_some_and(|stack| !stack.is_empty())
}

pub fn consume_placed_block_item(
    inventory: &mut InventoryState,
    gamemode: &PlayerGameMode,
) -> bool {
    if gamemode.is_creative() {
        return true;
    }

    let index = inventory.hotbar.active_index;
    let Some(stack) = inventory.hotbar.get_stack_mut(index) else {
        return false;
    };

    let _ = stack.take(1);
    if stack.is_empty() {
        inventory.hotbar.set_stack(index, ItemStack::empty());
    }
    true
}

pub fn can_break_block(
    block_id: u16,
    gamemode: &PlayerGameMode,
    tags: Option<&RuntimeTagRegistry>,
) -> bool {
    if block_id == 0 {
        return false;
    }
    if gamemode.is_creative() {
        return true;
    }
    !is_unbreakable_block(block_id, tags)
}

pub fn active_tool_data<'a>(
    active_stack: &ItemStack,
    item_registry: Option<&'a ItemRegistry>,
) -> Option<&'a ToolData> {
    if active_stack.is_empty() {
        return None;
    }

    item_registry
        .and_then(|registry| registry.get(active_stack.item_id()))
        .and_then(|definition| definition.tool_data())
}

pub fn can_harvest_block(block: &BlockProperty, active_tool: Option<&ToolData>) -> bool {
    let Some(required_tool) = block.required_tool else {
        return true;
    };

    let Some(tool) = active_tool else {
        return false;
    };

    tool.tool_type == required_tool && tool.tier.harvest_level() >= block.harvest_level
}

pub fn block_break_seconds(
    block: &BlockProperty,
    gamemode: &PlayerGameMode,
    active_tool: Option<&ToolData>,
) -> Option<f32> {
    if gamemode.is_creative() {
        return Some(0.0);
    }
    if block.hardness <= 0.0 {
        return Some(0.0);
    }
    if !can_harvest_block(block, active_tool) {
        return Some(
            (block.hardness
                * BASE_BREAK_SECONDS_PER_HARDNESS
                * INCORRECT_TOOL_BREAK_TIME_MULTIPLIER)
                .max(MIN_SURVIVAL_BREAK_SECONDS),
        );
    }

    let effective_tool = block.effective_tool.or(block.required_tool);
    let efficiency = match (effective_tool, active_tool) {
        (Some(effective_tool), Some(tool)) if tool.tool_type == effective_tool => tool.efficiency,
        _ => 1.0,
    }
    .max(MIN_TOOL_EFFICIENCY);

    Some(
        (block.hardness * BASE_BREAK_SECONDS_PER_HARDNESS / efficiency)
            .max(MIN_SURVIVAL_BREAK_SECONDS),
    )
}

pub fn is_unbreakable_block(block_id: u16, tags: Option<&RuntimeTagRegistry>) -> bool {
    tags.is_some_and(|tags| tags.contains(&block_tag(UNBREAKABLE_TAG), block_id))
}

pub fn is_replaceable_block(block_id: u16, tags: Option<&RuntimeTagRegistry>) -> bool {
    block_id == 0 || tags.is_some_and(|tags| tags.contains(&block_tag(REPLACEABLE_TAG), block_id))
}

fn block_tag(path: &str) -> TagId {
    TagId::new(BLOCK_TAG_NAMESPACE, path)
}
#[cfg(test)]
mod tests {
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
}
