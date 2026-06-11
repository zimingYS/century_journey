use crate::inventory::container::hotbar::HOTBAR_SIZE;
use crate::inventory::item::id::ItemId;
use crate::inventory::state::InventoryState;
use crate::ui::components::{HudHotbarContainer, HudRoot};
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{apply_item_texture, spawn_empty_slot, InventorySlot, SlotKind};
use crate::voxel::registry::BlockRegistry;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// 生成快捷栏UI
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

/// 更新快捷栏纹理并选中高亮
pub fn update_hotbar_ui_system(
    block_registry: Option<Res<BlockRegistry>>,
    state: Res<InventoryState>,
    mut commands: Commands,
    slot_query: Query<(Entity, &InventorySlot)>,
    children_query: Query<&Children>,
    mut image_node_query: Query<(&mut ImageNode, &mut BackgroundColor)>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    theme: Res<UiTheme>,
) {
    let Some(reg) = block_registry else { return; };

    // 更新快捷栏格子纹理
    for (entity, slot) in &slot_query {
        if slot.kind != SlotKind::Hotbar { continue; }
        let item_id = state.hotbar.items.get(slot.index)
            .cloned()
            .unwrap_or(ItemId::air());

        apply_item_texture(
            &mut commands, entity, &item_id, reg.as_ref(),
            &children_query, &mut image_node_query,
        );
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
    if state.opened { return; }

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
        if event.y > 0.0 { state.hotbar.select_prev(); }
        else { state.hotbar.select_next(); }
    }
}
