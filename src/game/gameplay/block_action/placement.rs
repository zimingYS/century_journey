use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::InventoryState;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;

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
