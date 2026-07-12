use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SearchInputState,
    SlotInteractionEvent, SlotKind,
};
use crate::game::inventory::events::DropItemEvent;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::EditableText;

/// 槽位左键或 Shift 左键交互。
pub fn slot_interaction_system(
    query: Query<(&Interaction, &InventorySlot), (Changed<Interaction>, With<Button>)>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SlotInteractionEvent>,
) {
    for (interaction, slot) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let action =
            if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
                SlotAction::ShiftClick
            } else if mouse.just_pressed(MouseButton::Left) {
                SlotAction::LeftClick
            } else {
                continue;
            };
        writer.write(SlotInteractionEvent {
            kind: slot.kind,
            index: slot.index,
            action,
        });
    }
}

/// 槽位右键交互。
pub fn slot_right_click_system(
    query: Query<(&Interaction, &InventorySlot), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SlotInteractionEvent>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
        return;
    }

    for (interaction, slot) in &query {
        if *interaction == Interaction::Hovered {
            writer.write(SlotInteractionEvent {
                kind: slot.kind,
                index: slot.index,
                action: SlotAction::RightClick,
            });
            break;
        }
    }
}

/// 背包打开时，悬停槽位并按 Q 丢弃物品。
pub fn slot_q_drop_system(
    query: Query<(&Interaction, &InventorySlot), With<Button>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SlotInteractionEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyQ) {
        return;
    }

    for (interaction, slot) in &query {
        if *interaction == Interaction::Hovered {
            let action =
                if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
                    SlotAction::DropAll
                } else {
                    SlotAction::DropOne
                };
            writer.write(SlotInteractionEvent {
                kind: slot.kind,
                index: slot.index,
                action,
            });
            break;
        }
    }
}

/// 正常游玩时，按 Q 直接丢弃当前快捷栏选中物品。
pub fn active_hotbar_q_drop_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    search_state: Res<SearchInputState>,
    mut inventory: ResMut<InventoryState>,
    mut drop_writer: MessageWriter<DropItemEvent>,
) {
    if inventory.opened || search_state.active || !keyboard.just_pressed(KeyCode::KeyQ) {
        return;
    }

    let take_count =
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            u32::MAX
        } else {
            1
        };

    let mut dropped_stack = None;
    {
        let slot = inventory.hotbar.active_stack_mut();
        if let Some(stack) = slot.as_mut() {
            dropped_stack = stack.take(take_count);
            if stack.is_empty() {
                *slot = None;
            }
        }
    }

    if let Some(stack) = dropped_stack {
        drop_writer.write(DropItemEvent { stack });
    }
}

/// 分类标签点击交互。
pub fn category_tab_interaction_system(
    mut query: Query<(&Interaction, &CategoryTab), (Changed<Interaction>, With<Button>)>,
    mut writer: MessageWriter<CategoryClickedEvent>,
) {
    for (interaction, tab) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        writer.write(CategoryClickedEvent {
            category_index: tab.category_index,
        });
    }
}

/// 同步 Bevy 输入焦点到项目原有 SearchInputState。
pub fn sync_search_input_focus_system(
    mut input_focus: ResMut<InputFocus>,
    input_query: Query<Entity, With<CreativeSearchInput>>,
    inventory: Res<InventoryState>,
    mut search_state: ResMut<SearchInputState>,
) {
    let focused_search = input_focus
        .get()
        .is_some_and(|entity| input_query.get(entity).is_ok());

    if !inventory.opened && focused_search {
        input_focus.clear();
        search_state.active = false;
        return;
    }

    search_state.active = inventory.opened && focused_search;
}

/// 把 EditableText 的真实文本同步到创造物品栏过滤条件。
pub fn sync_search_text_from_editable_system(
    mut inventory: ResMut<InventoryState>,
    query: Query<&EditableText, (With<CreativeSearchInput>, Changed<EditableText>)>,
) {
    let Ok(editable_text) = query.single() else {
        return;
    };

    let value = editable_text_value(editable_text);
    if inventory.creative.search_text != value {
        inventory.creative.search_text = value;
    }
}

/// 搜索框聚焦时，Escape 或 Enter 退出输入焦点。
pub fn search_escape_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input_focus: ResMut<InputFocus>,
    input_query: Query<Entity, With<CreativeSearchInput>>,
    mut search_state: ResMut<SearchInputState>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) && !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let Some(focused_entity) = input_focus.get() else {
        return;
    };
    if input_query.get(focused_entity).is_err() {
        return;
    }

    input_focus.clear();
    search_state.active = false;
}

/// 分类切换事件处理。
pub fn handle_category_clicked_system(
    mut reader: MessageReader<CategoryClickedEvent>,
    mut inventory: ResMut<InventoryState>,
) {
    for event in reader.read() {
        inventory.creative.selected_tab = event.category_index;
    }
}

/// 槽位点击和丢弃事件路由。
pub fn handle_slot_interaction_system(
    mut reader: MessageReader<SlotInteractionEvent>,
    mut inventory: ResMut<InventoryState>,
    mut drop_writer: MessageWriter<DropItemEvent>,
) {
    for event in reader.read() {
        if event.action == SlotAction::DropOne || event.action == SlotAction::DropAll {
            let take_count = if event.action == SlotAction::DropAll {
                u32::MAX
            } else {
                1
            };
            let container: &mut dyn crate::game::inventory::container::InventoryContainer =
                match event.kind {
                    SlotKind::Hotbar => &mut inventory.hotbar,
                    SlotKind::SurvivalBackpack
                    | SlotKind::SurvivalEquipment
                    | SlotKind::SurvivalAccessory => &mut inventory.survival,
                    _ => continue,
                };
            let event_index =
                crate::game::inventory::routing::survival_index(event.kind, event.index)
                    .unwrap_or(event.index);
            let (dropped_stack, emptied_slot) = {
                if let Some(slot_stack) = container.get_stack_mut(event_index) {
                    let dropped_stack = slot_stack.take(take_count);
                    (dropped_stack, slot_stack.is_empty())
                } else {
                    (None, false)
                }
            };

            if emptied_slot {
                container.set_stack(
                    event_index,
                    crate::game::inventory::item::stack::ItemStack::empty(),
                );
            }

            if let Some(stack) = dropped_stack {
                drop_writer.write(DropItemEvent { stack });
            }
            continue;
        }
        crate::game::inventory::routing::handle_slot_interaction(
            &mut inventory,
            event.kind,
            event.index,
            event.action,
        );
    }
}

/// 背包打开时，按 Escape 清空拖拽物品；搜索框聚焦时由搜索框先处理。
pub fn cancel_drag_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<InventoryState>,
    search_state: Res<SearchInputState>,
) {
    if !inventory.opened {
        return;
    }
    if search_state.active {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        inventory.cursor.clear();
    }
}

/// 槽位边框高亮。
pub fn slot_hover_system(
    theme: Res<UiTheme>,
    state: Res<InventoryState>,
    mut query: Query<
        (&InventorySlot, &Interaction, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (slot, interaction, mut border) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                *border = BorderColor::all(theme.border_hover);
            }
            Interaction::Pressed => {
                *border = BorderColor::all(theme.border_selected);
            }
            Interaction::None => {
                let selected =
                    slot.kind == SlotKind::Hotbar && slot.index == state.hotbar.active_index;
                *border = BorderColor::all(if selected {
                    theme.border_selected
                } else {
                    theme.border_default
                });
            }
        }
    }
}

/// 读取 EditableText 的值，忽略 IME 预编辑中的临时片段。
fn editable_text_value(editable_text: &EditableText) -> String {
    let mut value = String::new();
    value.reserve(editable_text.value().into_iter().map(str::len).sum());
    for part in editable_text.value() {
        value.push_str(part);
    }
    value
}
