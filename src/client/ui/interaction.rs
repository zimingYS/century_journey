use crate::client::ui::components::CreativeSearchBox;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SearchInputState,
    SlotInteractionEvent, SlotKind,
};
use crate::game::inventory::events::DropItemEvent;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

/// 槽位左键/Shift点击交互系统
/// 使用 `Changed<Interaction>` + Pressed（仅左键触发 Pressed）
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

/// 右键点击槽位系统（右键不触发 Pressed，需用 Hovered + mouse.just_pressed）
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
            break; // 一次只有一个槽位被 hover
        }
    }
}

/// Q 丢弃系统（需要持续检测 Hovered 状态，不能用 `Changed<Interaction>`）
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

/// 背包关闭时的快捷栏 Q 丢弃系统。
///
/// UI 槽位只负责在背包打开时处理悬停槽位；正常游玩时按 Q 应该直接丢弃当前快捷栏选中物品。
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

/// 分类标签点击交互系统。
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

/// 搜索框点击激活搜索框
pub fn search_box_interaction_system(
    mut query: Query<&Interaction, (Changed<Interaction>, With<CreativeSearchInput>)>,
    mut search_state: ResMut<SearchInputState>,
) {
    for interaction in &mut query {
        if *interaction == Interaction::Pressed {
            search_state.active = true;
        }
    }
}

/// 分类切换事件处理
pub fn handle_category_clicked_system(
    mut reader: MessageReader<CategoryClickedEvent>,
    mut inventory: ResMut<InventoryState>,
) {
    for event in reader.read() {
        inventory.creative.selected_tab = event.category_index;
    }
}

/// 槽位点击路由 (含 Q 丢弃)
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
                    SlotKind::SurvivalBackpack => &mut inventory.survival,
                    _ => continue,
                };
            let (dropped_stack, emptied_slot) = {
                if let Some(slot_stack) = container.get_stack_mut(event.index) {
                    let dropped_stack = slot_stack.take(take_count);
                    (dropped_stack, slot_stack.is_empty())
                } else {
                    (None, false)
                }
            };

            if emptied_slot {
                container.set_stack(
                    event.index,
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

/// 取消拖拽 — 仅 Escape 清空 cursor (右键不再清除)
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

/// 槽位边框高亮
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

// ═══════════════════════════════════════════════════════════
// 搜索系统
// ═══════════════════════════════════════════════════════════

pub fn search_keyboard_input_system(
    mut char_events: MessageReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<InventoryState>,
    search_state: Res<SearchInputState>,
) {
    if !search_state.active {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    if keyboard.just_pressed(KeyCode::Backspace) {
        inventory.creative.search_text.pop();
    }
    for ev in char_events.read() {
        let Some(text) = &ev.text else { continue };
        for ch in text.chars() {
            if !ch.is_control() {
                inventory.creative.search_text.push(ch);
            }
        }
    }
}

pub fn search_escape_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut search_state: ResMut<SearchInputState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Enter) {
        search_state.active = false;
    }
}

pub fn update_search_text_display_system(
    inventory: Res<InventoryState>,
    query: Query<&Children, With<CreativeSearchBox>>,
    mut text_query: Query<&mut Text>,
) {
    let Ok(children) = query.single() else { return };
    let search = inventory.creative.search_text.clone();
    for child in children.iter() {
        if let Ok(mut text) = text_query.get_mut(child) {
            *text = Text::new(if search.is_empty() {
                "🔍 搜索...".into()
            } else {
                search.clone()
            });
        }
    }
}
