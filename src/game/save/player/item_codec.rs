use crate::content::item::ItemRegistry;
use crate::game::inventory::item::stack::{ItemInstanceData, ItemStack};
use crate::game::save::player::SaveItemStack;
use crate::shared::item_id::ItemId;

fn item_id_to_string(id: &ItemId) -> String {
    id.to_string()
}

fn string_to_item_id(s: &str) -> ItemId {
    if let Some(rest) = s.strip_prefix("item:") {
        ItemId::item(rest)
    } else if let Some(rest) = s.strip_prefix("block:") {
        ItemId::block(rest)
    } else {
        ItemId::block(s)
    }
}

pub(in crate::game) fn optional_stack_to_save(
    opt: Option<&ItemStack>,
    item_registry: &ItemRegistry,
) -> SaveItemStack {
    match opt {
        Some(s) if !s.is_empty() => SaveItemStack {
            runtime_id: item_registry.runtime_id(&s.item),
            item: item_id_to_string(&s.item),
            count: s.count,
            durability: s.instance.durability,
        },
        _ => SaveItemStack::air(),
    }
}

pub(in crate::game) fn save_to_optional_stack(slot: &SaveItemStack) -> Option<ItemStack> {
    if slot.is_air() {
        None
    } else {
        Some(ItemStack::with_instance(
            string_to_item_id(&slot.item),
            slot.count,
            ItemInstanceData {
                durability: slot.durability,
            },
        ))
    }
}

pub(in crate::game) fn save_to_optional_stack_with_registry(
    slot: &SaveItemStack,
    item_registry: &ItemRegistry,
    remap: &std::collections::HashMap<u32, u32>,
) -> Option<ItemStack> {
    if slot.is_air() {
        return None;
    }
    let item = string_to_item_id(&slot.item);
    if !item_registry.contains(&item) {
        log::warn!(
            "[存档系统] 物品 {} 在当前内容版本中不存在，已将槽位清空",
            slot.item
        );
        return None;
    }
    if let Some(saved_runtime_id) = slot.runtime_id {
        let current_runtime_id = item_registry.runtime_id(&item);
        match (remap.get(&saved_runtime_id), current_runtime_id) {
            (Some(mapped), Some(current)) if *mapped == current => {
                if saved_runtime_id != current {
                    log::info!(
                        "[存档系统] 物品 {} 动态 ID 已从 {} 重映射为 {}",
                        slot.item,
                        saved_runtime_id,
                        current
                    );
                }
            }
            _ => log::warn!(
                "[存档系统] 物品 {} 的旧动态 ID {} 无法可信重映射，改用唯一标识符恢复",
                slot.item,
                saved_runtime_id
            ),
        }
    }
    Some(ItemStack::with_instance(
        item,
        slot.count,
        ItemInstanceData {
            durability: slot.durability,
        },
    ))
}
