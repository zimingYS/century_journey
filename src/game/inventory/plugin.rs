use bevy::prelude::*;

use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::events::{
    DropItemEvent, InventoryCommand, InventoryFeedbackEvent, SlotInteractionEvent,
};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use crate::shared::ui_types::SlotKind;

/// Game 层 Inventory 模块 Plugin。
///
/// 只负责 Game 层运行时系统。
/// Definition/Registry/Loader/Texture 已在 Content 层的 ItemContentPlugin 中注册。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryState>()
            .init_resource::<crate::game::inventory::equipment::AccessorySlotDefinitions>()
            .add_message::<SlotInteractionEvent>()
            .add_message::<DropItemEvent>()
            .add_message::<InventoryCommand>()
            .add_message::<InventoryFeedbackEvent>()
            .add_systems(
                Update,
                (
                    handle_slot_interaction_system,
                    handle_inventory_command_system,
                ),
            );
    }
}

fn handle_inventory_command_system(
    mut reader: MessageReader<InventoryCommand>,
    mut inventory: ResMut<InventoryState>,
) {
    for command in reader.read() {
        match command {
            InventoryCommand::CompactBackpack => compact_backpack(&mut inventory.survival.backpack),
            InventoryCommand::SortBackpack => sort_backpack(&mut inventory.survival.backpack),
        }
    }
}

fn compact_backpack(
    backpack: &mut [Option<ItemStack>;
        crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE],
) {
    let compacted: Vec<ItemStack> = backpack
        .iter_mut()
        .filter_map(Option::take)
        .filter(|stack| !stack.is_empty())
        .collect();
    for (index, slot) in backpack.iter_mut().enumerate() {
        *slot = compacted.get(index).cloned();
    }
}

fn sort_backpack(
    backpack: &mut [Option<ItemStack>;
        crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE],
) {
    let mut stacks: Vec<ItemStack> = backpack
        .iter_mut()
        .filter_map(Option::take)
        .filter(|stack| !stack.is_empty())
        .collect();
    stacks.sort_by_key(|stack| stack.item.to_string());

    let mut packed: Vec<ItemStack> = Vec::with_capacity(stacks.len());
    for mut incoming in stacks {
        for existing in &mut packed {
            if existing.is_same_item(&incoming) && !existing.is_full() {
                existing.merge_from(&mut incoming);
            }
            if incoming.is_empty() {
                break;
            }
        }
        if !incoming.is_empty() {
            packed.push(incoming);
        }
    }
    for (index, slot) in backpack.iter_mut().enumerate() {
        *slot = packed.get(index).cloned();
    }
}

fn handle_slot_interaction_system(
    mut reader: MessageReader<SlotInteractionEvent>,
    mut inventory: ResMut<InventoryState>,
    mut drop_writer: MessageWriter<DropItemEvent>,
) {
    for event in reader.read() {
        if matches!(event.action, SlotAction::DropOne | SlotAction::DropAll) {
            drop_from_slot(event, &mut inventory, &mut drop_writer);
            continue;
        }
        crate::game::inventory::routing::handle_slot_interaction(
            &mut inventory,
            event.kind,
            event.index,
            event.action,
        );
    }
}

fn drop_from_slot(
    event: &SlotInteractionEvent,
    inventory: &mut InventoryState,
    drop_writer: &mut MessageWriter<DropItemEvent>,
) {
    let take_count = if event.action == SlotAction::DropAll {
        u32::MAX
    } else {
        1
    };
    let container: &mut dyn InventoryContainer = match event.kind {
        SlotKind::Hotbar => &mut inventory.hotbar,
        SlotKind::SurvivalBackpack | SlotKind::SurvivalEquipment | SlotKind::SurvivalAccessory => {
            &mut inventory.survival
        }
        _ => return,
    };
    let index = crate::game::inventory::routing::survival_index(event.kind, event.index)
        .unwrap_or(event.index);
    let Some(slot_stack) = container.get_stack_mut(index) else {
        return;
    };
    let dropped = slot_stack.take(take_count);
    let emptied = slot_stack.is_empty();
    if emptied {
        container.set_stack(index, ItemStack::empty());
    }
    if let Some(stack) = dropped {
        drop_writer.write(DropItemEvent { stack });
    }
}
