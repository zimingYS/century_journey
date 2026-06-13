use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use crate::inventory::slot::SlotAction;
use crate::inventory::state::InventoryState;
use crate::ui::components::CreativeSearchBox;
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SearchInputState, SlotInteractionEvent, SlotKind};

/// 槽位点击交互系统
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

        let action = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            SlotAction::ShiftClick
        } else if mouse.just_pressed(MouseButton::Left) {
            SlotAction::LeftClick
        } else if mouse.just_pressed(MouseButton::Right) {
            SlotAction::RightClick
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

/// 分类标签点击交互系统
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

/// 槽位点击路由
pub fn handle_slot_interaction_system(
    mut reader: MessageReader<SlotInteractionEvent>,
    mut inventory: ResMut<InventoryState>,
) {
    for event in reader.read() {
        crate::inventory::interaction::handle_slot_interaction(
            &mut inventory, event.kind, event.index, event.action,
        );
    }
}

/// 取消拖拽
pub fn cancel_drag_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut inventory: ResMut<InventoryState>,
    search_state: Res<SearchInputState>,
) {
    if !inventory.opened { return; }
    if search_state.active { return; }

    if keyboard.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
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
                let selected = slot.kind == SlotKind::Hotbar
                    && slot.index == state.hotbar.active_index;
                *border = BorderColor::all(if selected {
                    theme.border_selected
                } else {
                    theme.border_default
                });
            }
        }
    }
}


//  **搜索系统**
/// 搜索键盘输入系统
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

/// 退出搜索框系统
pub fn search_escape_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut search_state: ResMut<SearchInputState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Enter)
    {
        search_state.active = false;
    }
}

/// 更新搜索框显示文本
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