//! 消费槽位交互消息并更新容器或生成世界掉落请求。

use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::events::{DropItemEvent, SlotInteractionEvent};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::slot::SlotKind;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::PlayerId;
use bevy::prelude::{MessageReader, MessageWriter, Query};

/// 在固定步路由玩家槽位操作，并把丢弃结果发布为领域消息。
pub fn handle_slot_interaction_system(
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
        crate::game::inventory::interaction::routing::handle_slot_interaction(
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
    let index =
        crate::game::inventory::interaction::routing::survival_index(event.kind, event.index)
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
