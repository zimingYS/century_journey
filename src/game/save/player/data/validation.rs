//! 校验玩家存档的容量、索引和数值范围，拒绝损坏数据。

use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::save::player::PlayerSaveData;
use crate::game::save::player::SaveItemStack;
use crate::game::save::player::data::model;
use crate::shared::item_id::ItemId;

/// 存档数据健康检查与自动修复
pub(in crate::game) fn validate_player_data(data: &PlayerSaveData) -> PlayerSaveData {
    let mut data = data.clone();
    let mut repaired = false;

    data.item_id_map.sort_by_key(|(runtime_id, _)| *runtime_id);
    let mut seen_runtime_ids = std::collections::HashSet::new();
    let mut seen_identifiers = std::collections::HashSet::new();
    data.item_id_map.retain(|(runtime_id, identifier)| {
        let valid = ItemId::parse(identifier).is_ok()
            && seen_runtime_ids.insert(*runtime_id)
            && seen_identifiers.insert(identifier.clone());
        if !valid {
            log::warn!(
                "[存档系统] 无效或重复的物品 ID 映射: {} -> {}，已移除",
                runtime_id,
                identifier
            );
            repaired = true;
        }
        valid
    });

    if data.position.iter().any(|v| v.is_nan() || v.is_infinite()) {
        log::warn!("[存档系统] 无效位置{:?}，已重置为世界原点", data.position);
        data.position = [0.0, 70.0, 0.0];
        repaired = true;
    }
    if data.rotation.iter().any(|v| v.is_nan() || v.is_infinite()) {
        log::warn!("[存档系统] 旋转无效 {:?}, 已重置为恒等矩阵", data.rotation);
        data.rotation = [0.0, 0.0, 0.0, 1.0];
        repaired = true;
    }
    if data.camera_pitch.is_nan() || data.camera_pitch.is_infinite() {
        log::warn!(
            "[存档系统] 相机俯仰角{}无效, 已重置为0.0",
            data.camera_pitch
        );
        data.camera_pitch = 0.0;
        repaired = true;
    }
    if !data.health.is_finite() {
        data.health = 20.0;
        repaired = true;
    } else {
        data.health = data.health.clamp(0.0, 20.0);
    }
    if !data.hunger.is_finite() {
        data.hunger = 20.0;
        repaired = true;
    } else {
        data.hunger = data.hunger.clamp(0.0, 20.0);
    }
    if !data.saturation.is_finite() {
        data.saturation = model::default_saturation();
        repaired = true;
    } else {
        data.saturation = data.saturation.clamp(0.0, data.hunger);
    }
    if data.respawn_point.iter().any(|value| !value.is_finite()) {
        data.respawn_point = model::default_respawn_point();
        repaired = true;
    }
    if !matches!(data.gamemode.as_str(), "survival" | "creative") {
        log::warn!(
            "[存档系统] 未知游戏模式: '{}', 已重置为生存模式",
            data.gamemode
        );
        data.gamemode = "survival".into();
        repaired = true;
    }
    if data.hotbar_active >= HOTBAR_SIZE {
        log::warn!(
            "[存档系统] 快捷栏索引 {} 超出索引范围,已重置为0",
            data.hotbar_active
        );
        data.hotbar_active = 0;
        repaired = true;
    }
    for (slot, kind) in data
        .hotbar
        .iter_mut()
        .map(|s| (s, "hotbar"))
        .chain(data.backpack.iter_mut().map(|s| (s, "backpack")))
        .chain(data.equipment.iter_mut().map(|s| (s, "equipment")))
        .chain(data.accessories.iter_mut().map(|s| (s, "accessories")))
    {
        if slot.is_air() {
            continue;
        }
        if slot.item.is_empty() || !slot.item.contains(':') {
            log::warn!(
                "[存档系统] '{}'中的物品{}无效,已替换为空气",
                slot.item,
                kind
            );
            *slot = SaveItemStack::air();
            repaired = true;
            continue;
        }
        if let Some(runtime_id) = slot.runtime_id
            && !data
                .item_id_map
                .iter()
                .any(|(mapped_id, identifier)| *mapped_id == runtime_id && identifier == &slot.item)
        {
            log::warn!(
                "[存档系统] {kind} 中物品 {} 的动态 ID {} 与映射表不一致，将按唯一标识符恢复",
                slot.item,
                runtime_id
            );
            slot.runtime_id = None;
            repaired = true;
        }
    }

    if repaired {
        log::warn!("[存档系统] 保存数据出现问题 — 已自动修复");
    }
    data
}
