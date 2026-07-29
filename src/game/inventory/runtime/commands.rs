use crate::game::inventory::events::InventoryCommand;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::LocalPlayer;
use bevy::prelude::{MessageReader, Query, With};

pub fn handle_inventory_command_system(
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
