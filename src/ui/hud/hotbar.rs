use crate::inventory::container::hotbar::HOTBAR_SIZE;
use crate::inventory::item::id::ItemId;
use crate::inventory::state::InventoryState;
use crate::ui::components::{HudHotbarContainer, HudRoot};
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{
    spawn_empty_slot, sync_slot_icon, InventorySlot, SlotKind, SlotVisual,
};
use crate::voxel::registry::BlockRegistry;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// 生成HUD快捷栏
pub fn spawn_hotbar_ui_system(mut commands: Commands, theme: Res<UiTheme>) {
    commands.spawn((
        HudRoot,
        HudHotbarContainer,
        Name::new("HudHotbar"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(35.0),
            width: Val::Percent(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(theme.slot_gap),
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(theme.hotbar_bg),
        BorderColor::all(theme.border_default),
    )).with_children(|parent| {
        for index in 0..HOTBAR_SIZE {
            spawn_empty_slot(parent, SlotKind::Hotbar, index, &theme);
        }
    });
}

/// HUD快捷栏视觉同步
pub fn hud_hotbar_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    mut commands: Commands,
    slot_query: Query<(Entity, &InventorySlot)>,
    mut slot_visual_query: Query<&mut SlotVisual>,
    children_query: Query<&Children>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    theme: Res<UiTheme>,
    mut last_hotbar: Local<Vec<ItemId>>,
) {
    let Some(reg) = block_registry.as_ref() else { return };
    let current: Vec<ItemId> = state.hotbar.items.to_vec();

    // 热栏未变则跳过
    if *last_hotbar == current { return; }
    *last_hotbar = current.clone();

    // 原地更新图标 + 回写 SlotVisual
    for (entity, slot) in &slot_query {
        if slot.kind != SlotKind::Hotbar { continue; }
        let item = current.get(slot.index).cloned().unwrap_or(ItemId::air());

        if let Ok(mut visual) = slot_visual_query.get_mut(entity) {
            if visual.item != item {
                sync_slot_icon(&mut commands, entity, &item, reg, &children_query);
                visual.item = item;       
            }
        }
    }

    // 更新选中边框
    for (slot, mut border) in &mut border_query {
        if slot.kind != SlotKind::Hotbar { continue; }
        *border = BorderColor::all(
            if slot.index == state.hotbar.active_index {
                theme.border_selected
            } else {
                theme.border_default
            }
        );
    }
}

/// 数字键/滚轮切换快捷栏
pub fn handle_hotbar_switch_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut state: ResMut<InventoryState>,
) {
    if state.opened {
        return;
    }

    let num_keys = [
        (KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3), (KeyCode::Digit5, 4), (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6), (KeyCode::Digit8, 7), (KeyCode::Digit9, 8),
    ];
    for (key, idx) in num_keys {
        if keyboard.just_pressed(key) {
            state.hotbar.active_index = idx;
            return;
        }
    }

    for event in mouse_wheel.read() {
        if event.y > 0.0 {
            state.hotbar.select_prev();
        } else {
            state.hotbar.select_next();
        }
    }
}