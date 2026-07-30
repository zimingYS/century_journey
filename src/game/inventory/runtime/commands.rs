//! 处理来自 Client 的本地物品栏命令。
//!
//! 所有会改变权威库存的操作都在固定步执行；关闭背包时无法归还的光标物品会转成
//! `DropItemEvent`，保证任何容量边界下都不会静默丢失物品。

use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::events::{DropItemEvent, InventoryCommand};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::{CursorSource, InventoryState};
use crate::game::player::identity::{LocalPlayer, PlayerId};
use bevy::prelude::{MessageReader, MessageWriter, Query, Res, With};

/// 在固定步中消费本地物品栏命令。
pub fn handle_inventory_command_system(
    mut reader: MessageReader<InventoryCommand>,
    gamemode: Res<PlayerGameMode>,
    mut inventory_query: Query<(&PlayerId, &mut InventoryState), With<LocalPlayer>>,
    mut drop_writer: MessageWriter<DropItemEvent>,
) {
    let Ok((player_id, mut inventory)) = inventory_query.single_mut() else {
        return;
    };
    for command in reader.read() {
        match command {
            InventoryCommand::Open => inventory.opened = true,
            InventoryCommand::Close => close_inventory(
                *player_id,
                &mut inventory,
                gamemode.is_creative(),
                &mut drop_writer,
            ),
            InventoryCommand::Toggle if inventory.opened => close_inventory(
                *player_id,
                &mut inventory,
                gamemode.is_creative(),
                &mut drop_writer,
            ),
            InventoryCommand::Toggle => inventory.opened = true,
            InventoryCommand::CompactBackpack => compact_backpack(&mut inventory.survival.backpack),
            InventoryCommand::SortBackpack => sort_backpack(&mut inventory.survival.backpack),
        }
    }
}

fn close_inventory(
    player_id: PlayerId,
    inventory: &mut InventoryState,
    creative: bool,
    drop_writer: &mut MessageWriter<DropItemEvent>,
) {
    inventory.opened = false;
    if creative {
        inventory.cursor.clear();
        return;
    }

    if let Some(stack) = return_cursor_item(inventory) {
        drop_writer.write(DropItemEvent { player_id, stack });
    }
}

/// 尝试把光标物品依次归还到来源、当前快捷栏、其他快捷栏和主背包。
///
/// 返回值是所有候选槽位都无法容纳的剩余物品，调用方必须将其掉落或继续保存，不能丢弃。
fn return_cursor_item(inventory: &mut InventoryState) -> Option<ItemStack> {
    let source = inventory.cursor.source;
    let mut remaining = inventory.cursor.take_stack()?;

    if let Some(source) = source {
        match source {
            CursorSource::Hotbar(index) => {
                remaining = return_to_container(&mut inventory.hotbar, index, remaining);
            }
            CursorSource::SurvivalBackpack(index) => {
                remaining = return_to_container(&mut inventory.survival, index, remaining);
            }
            CursorSource::CreativeGrid(_)
            | CursorSource::Recent(_)
            | CursorSource::Container(_) => {}
        }
    }

    let active = inventory.hotbar.active_index;
    remaining = return_to_container(&mut inventory.hotbar, active, remaining);
    for index in 0..HOTBAR_SIZE {
        remaining = return_to_container(&mut inventory.hotbar, index, remaining);
        if remaining.is_empty() {
            return None;
        }
    }
    for index in 0..SurvivalInventory::BACKPACK_SIZE {
        remaining = return_to_container(&mut inventory.survival, index, remaining);
        if remaining.is_empty() {
            return None;
        }
    }

    (!remaining.is_empty()).then_some(remaining)
}

fn return_to_container<C: InventoryContainer>(
    container: &mut C,
    index: usize,
    mut remaining: ItemStack,
) -> ItemStack {
    if remaining.is_empty() {
        return remaining;
    }
    if let Some(stack) = container.get_stack_mut(index)
        && stack.is_same_item(&remaining)
    {
        stack.merge_from(&mut remaining);
    }
    if !remaining.is_empty() && container.get_stack(index).is_none() {
        container.set_stack(index, remaining);
        return ItemStack::empty();
    }
    remaining
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

#[cfg(test)]
#[path = "../../../../tests/unit/game/inventory/runtime/commands.rs"]
mod tests;
