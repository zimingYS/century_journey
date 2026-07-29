use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::CursorData;
use std::ops::Range;

// 鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?// 鏍稿績浜や簰鍑芥暟 鈥?绾暟鎹搷浣滐紝鏃?UI 渚濊禆
// 鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?
/// 宸﹂敭鐐瑰嚮妲戒綅
///
/// 瀹炵幇 Minecraft 鏍囧噯琛屼负锛?/// - 鍏夋爣绌?+ 妲芥湁鐗?鈫?鎷胯捣鍏ㄩ儴
/// - 鍏夋爣鏈夌墿 + 妲界┖ 鈫?鏀句笅鍏ㄩ儴
/// - 鍏夋爣鏈夌墿 + 妲芥湁鍚岀 鈫?鍚堝苟锛堣秴鍑虹暀鍦ㄥ厜鏍囷級
/// - 鍏夋爣鏈夌墿 + 妲芥湁涓嶅悓 鈫?浜ゆ崲
pub fn left_click_slot<C: InventoryContainer>(
    container: &mut C,
    index: usize,
    cursor: &mut CursorData,
) {
    let slot_has = container.get_stack(index).is_some_and(|s| !s.is_empty());
    let cursor_has = cursor.has_item();

    match (cursor_has, slot_has) {
        (false, true) => {
            if let Some(stack) = container.replace_stack(index, ItemStack::empty()) {
                cursor.set_stack(stack);
            }
        }
        (true, false) => {
            if let Some(stack) = cursor.take_stack() {
                container.set_stack(index, stack);
            }
        }
        (true, true) => {
            let Some(slot_stack) = container.get_stack(index) else {
                return;
            };
            let Some(cursor_stack) = cursor.stack() else {
                return;
            };
            let is_same = cursor_stack.item == slot_stack.item;

            if is_same {
                if let Some(slot_stack) = container.get_stack_mut(index)
                    && let Some(cursor_stack) = cursor.stack_mut()
                {
                    slot_stack.merge_from(cursor_stack);
                }

                // 濡傛灉鍏夋爣绌猴紝娓呴櫎鍏夋爣
                if cursor.stack().is_none_or(|s| s.is_empty()) {
                    cursor.clear();
                }
            } else {
                if let Some(slot_stack) = container.replace_stack(index, ItemStack::empty()) {
                    let cursor_stack = cursor.take_stack().unwrap_or_default();
                    cursor.set_stack(slot_stack);
                    container.set_stack(index, cursor_stack);
                }
            }
        }
        (false, false) => {}
    }
}

/// 鍙抽敭鐐瑰嚮妲戒綅
///
/// 瀹炵幇 Minecraft 鏍囧噯琛屼负锛?/// - 鍏夋爣绌?+ 妲芥湁鐗?鈫?鎷胯蛋涓€鍗婏紙濂囨暟鍚戜笂鍙栨暣锛?/// - 鍏夋爣鏈夌墿 + 妲界┖ 鈫?鏀惧叆 1 涓?/// - 鍏夋爣鏈夌墿 + 妲芥湁鍚岀涓旀湭婊?鈫?鏀惧叆 1 涓?/// - 涓嶅悓鐗╁搧 鈫?鏃犳搷浣?
pub fn right_click_slot<C: InventoryContainer>(
    container: &mut C,
    index: usize,
    cursor: &mut CursorData,
) {
    let slot_has = container.get_stack(index).is_some_and(|s| !s.is_empty());
    let cursor_has = cursor.has_item();

    match (cursor_has, slot_has) {
        (false, true) => {
            let Some(stack) = container.get_stack(index) else {
                return;
            };
            let total = stack.count;
            let half = total.div_ceil(2);
            let remaining = total - half;

            if remaining == 0 {
                if let Some(stack) = container.replace_stack(index, ItemStack::empty()) {
                    cursor.set_stack(stack);
                }
            } else {
                if let Some(slot_stack) = container.get_stack_mut(index) {
                    slot_stack.count = remaining;
                }

                let Some(stack) = container.get_stack(index) else {
                    return;
                };
                let cursor_stack = ItemStack::new(stack.item.clone(), half);
                cursor.set_stack(cursor_stack);
            }
        }
        (true, false) => {
            let Some(cursor_stack) = cursor.stack() else {
                return;
            };
            let cursor_count = cursor_stack.count;
            let take = 1.min(cursor_count);

            let mut new_cursor = cursor_stack.clone();
            new_cursor.count = cursor_count - take;

            let mut new_slot = cursor_stack.clone();
            new_slot.count = take;

            if new_cursor.count == 0 {
                cursor.take_stack();
            } else {
                cursor.set_stack(new_cursor);
            }
            container.set_stack(index, new_slot);
        }
        (true, true) => {
            let Some(slot) = container.get_stack(index) else {
                return;
            };
            let Some(cursor_item) = cursor.stack() else {
                return;
            };
            let is_same = cursor_item.item == slot.item;

            if is_same && slot.count < ItemStack::MAX_STACK_SIZE {
                if let Some(slot_stack) = container.get_stack_mut(index) {
                    slot_stack.count += 1;
                }
                if let Some(cursor_stack) = cursor.stack_mut() {
                    cursor_stack.count -= 1;
                    if cursor_stack.count == 0 {
                        cursor.take_stack();
                    }
                }
            }
        }
        (false, false) => {}
    }
}

/// Shift + 鐐瑰嚮妲戒綅锛堝揩閫熻浆绉伙級
///
/// 鍦?source 鍜?dest 瀹瑰櫒闂磋浆绉荤墿鍝侊細
/// 1. 浼樺厛鍚堝苟鍒?dest 涓凡鏈夌殑鍚岀鍫嗗彔
/// 2. 鍐嶅鎵?dest 涓涓€涓┖妲戒綅鏀惧叆
pub fn shift_click<C1: InventoryContainer, C2: InventoryContainer>(
    source: &mut C1,
    dest: &mut C2,
    index: usize,
) -> bool {
    let slot_count = dest.slot_count();
    shift_click_into_range(source, dest, index, 0..slot_count)
}

/// Shift 鐐瑰嚮骞舵妸鐗╁搧闄愬埗鍦ㄧ洰鏍囧鍣ㄧ殑鎸囧畾妲戒綅鑼冨洿鍐呫€?
pub fn shift_click_into_range<C1: InventoryContainer, C2: InventoryContainer>(
    source: &mut C1,
    dest: &mut C2,
    index: usize,
    range: Range<usize>,
) -> bool {
    let Some(source_stack) = source.get_stack(index) else {
        return false;
    };
    if source_stack.is_empty() {
        return false;
    }

    let mut remaining = source_stack.clone();

    // 绗竴姝ワ細浼樺厛鍚堝苟鍒板凡鏈夊悓绫诲爢鍙犮€?
    for i in range.clone() {
        if remaining.is_empty() {
            break;
        }
        if let Some(dest_stack) = dest.get_stack_mut(i)
            && dest_stack.is_same_item(&remaining)
        {
            dest_stack.merge_from(&mut remaining);
        }
    }

    // 绗簩姝ワ細灏嗗墿浣欑墿鍝佹斁鍏ョ涓€涓┖妲戒綅銆?
    if !remaining.is_empty() {
        for i in range {
            if dest.get_stack(i).is_none_or(|s| s.is_empty()) {
                dest.set_stack(i, remaining.clone());
                remaining = ItemStack::empty();
                break;
            }
        }
    }

    if remaining.is_empty() {
        source.replace_stack(index, ItemStack::empty());
        true
    } else {
        let moved_count = source_stack.count - remaining.count;
        if moved_count > 0 {
            let mut updated = source_stack.clone();
            updated.count = remaining.count;
            source.set_stack(index, updated);
            true
        } else {
            false
        }
    }
}

/// 灏嗘潵婧愭Ы浣嶄腑鐨勪竴涓墿鍝佺Щ鍔ㄥ埌鐩爣鑼冨洿锛屼緵婊氳疆蹇€熻浆绉讳娇鐢ㄣ€?
pub fn move_one_into_range<C1: InventoryContainer, C2: InventoryContainer>(
    source: &mut C1,
    dest: &mut C2,
    source_index: usize,
    dest_range: Range<usize>,
) -> bool {
    let Some(source_stack) = source.get_stack(source_index).cloned() else {
        return false;
    };
    if source_stack.is_empty() {
        return false;
    }
    let mut one =
        ItemStack::with_instance(source_stack.item.clone(), 1, source_stack.instance.clone());
    if !insert_one_into_range(dest, &mut one, dest_range) {
        return false;
    }

    let emptied = source
        .get_stack_mut(source_index)
        .and_then(|stack| stack.take(1).map(|_| stack.is_empty()))
        .unwrap_or(false);
    if emptied {
        source.set_stack(source_index, ItemStack::empty());
    }
    true
}

/// 浠庢潵婧愯寖鍥村鎵句笌鐩爣妲戒綅鐩稿悓鐨勭墿鍝侊紝骞惰ˉ鍏ヤ竴涓€?
pub fn pull_one_matching<C1: InventoryContainer, C2: InventoryContainer>(
    dest: &mut C1,
    source: &mut C2,
    dest_index: usize,
    source_range: Range<usize>,
) -> bool {
    let Some(target) = dest.get_stack(dest_index).cloned() else {
        return false;
    };
    if target.is_empty() || target.is_full() {
        return false;
    }
    let source_index = source_range.into_iter().find(|index| {
        source
            .get_stack(*index)
            .is_some_and(|stack| stack.is_same_item(&target) && !stack.is_empty())
    });
    source_index.is_some_and(|index| {
        move_one_into_range(
            source,
            dest,
            index,
            dest_index..dest_index.saturating_add(1),
        )
    })
}

fn insert_one_into_range<C: InventoryContainer>(
    dest: &mut C,
    one: &mut ItemStack,
    range: Range<usize>,
) -> bool {
    for index in range.clone() {
        if let Some(existing) = dest.get_stack_mut(index)
            && existing.is_same_item(one)
            && !existing.is_full()
        {
            existing.merge_from(one);
            return one.is_empty();
        }
    }
    for index in range {
        if dest.get_stack(index).is_none_or(ItemStack::is_empty) {
            dest.set_stack(index, std::mem::take(one));
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/inventory/click.rs"]
mod tests;
