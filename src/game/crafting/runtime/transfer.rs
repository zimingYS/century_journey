use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;

pub(super) fn capacity_range<C: InventoryContainer + ?Sized>(
    container: &C,
    incoming: &ItemStack,
    range: std::ops::Range<usize>,
) -> u32 {
    range
        .map(|index| match container.get_stack(index) {
            Some(stack) if stack.item == incoming.item => stack.remaining_space(),
            None => ItemStack::MAX_STACK_SIZE,
            _ => 0,
        })
        .sum()
}

pub(super) fn insert_range<C: InventoryContainer + ?Sized>(
    container: &mut C,
    incoming: &mut ItemStack,
    range: std::ops::Range<usize>,
) {
    for index in range.clone() {
        if incoming.is_empty() {
            return;
        }
        if let Some(stack) = container.get_stack_mut(index)
            && stack.item == incoming.item
        {
            stack.merge_from(incoming);
        }
    }
    for index in range {
        if incoming.is_empty() {
            return;
        }
        if container.get_stack(index).is_none_or(ItemStack::is_empty) {
            container.set_stack(index, std::mem::take(incoming));
        }
    }
}
