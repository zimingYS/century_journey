use bevy::prelude::*;

use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::world::WorldContainers;
use crate::game::inventory::events::{
    DropItemEvent, InventoryCommand, InventoryFeedbackEvent, SlotInteractionEvent,
};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::command::apply_player_command_system;
use crate::game::player::components::{LocalPlayer, PlayerId};
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use crate::shared::ui_types::SlotKind;

/// Game 层 Inventory 模块 Plugin。
///
/// 只负责 Game 层运行时系统。
/// Definition/Registry/Loader/Texture 已在 Content 层的 ItemContentPlugin 中注册。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::game::inventory::equipment::AccessorySlotDefinitions>()
            .init_resource::<WorldContainers>()
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
            )
            .add_systems(
                FixedUpdate,
                handle_hotbar_command_system
                    .after(apply_player_command_system)
                    .in_set(SimulationSet::Commands)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn handle_hotbar_command_system(
    actions: Res<PlayerActionState>,
    mut inventory_query: Query<&mut InventoryState, With<LocalPlayer>>,
) {
    let Ok(mut inventory) = inventory_query.single_mut() else {
        return;
    };
    let direct = [
        PlayerAction::Hotbar1,
        PlayerAction::Hotbar2,
        PlayerAction::Hotbar3,
        PlayerAction::Hotbar4,
        PlayerAction::Hotbar5,
        PlayerAction::Hotbar6,
        PlayerAction::Hotbar7,
        PlayerAction::Hotbar8,
        PlayerAction::Hotbar9,
    ];
    for (index, action) in direct.into_iter().enumerate() {
        if actions.just_pressed(action) {
            inventory.hotbar.active_index = index;
            return;
        }
    }
    if actions.just_pressed(PlayerAction::HotbarPrevious) {
        inventory.hotbar.select_prev();
    }
    if actions.just_pressed(PlayerAction::HotbarNext) {
        inventory.hotbar.select_next();
    }
}

fn handle_inventory_command_system(
    mut reader: MessageReader<InventoryCommand>,
    mut inventory_query: Query<&mut InventoryState, With<LocalPlayer>>,
) {
    let Ok(mut inventory) = inventory_query.single_mut() else {
        return;
    };
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
    mut inventories: Query<(&PlayerId, &mut InventoryState)>,
    mut drop_writer: MessageWriter<DropItemEvent>,
) {
    for event in reader.read() {
        let Some((_, mut inventory)) = inventories
            .iter_mut()
            .find(|(player_id, _)| **player_id == event.player_id)
        else {
            continue;
        };
        if matches!(event.kind, SlotKind::Container(_)) {
            continue;
        }
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
        drop_writer.write(DropItemEvent {
            player_id: event.player_id,
            stack,
        });
    }
}
