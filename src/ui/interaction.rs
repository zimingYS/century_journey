use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::inventory::state::InventoryState;
use crate::ui::components::CreativeSearchBox;
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SearchInputState, SlotClickedEvent, SlotKind};

/// 槽位交互系统
pub fn slot_interaction_system(
    query: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>)
    >,
    mut writer: MessageWriter<SlotClickedEvent>,
) {
    for (interaction, slot) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        writer.write(
            SlotClickedEvent {
                kind: slot.kind,
                index: slot.index,
            }
        );
    }
}


/// 槽位边框高亮
pub fn slot_hover_system(
    theme: Res<UiTheme>,
    state: Res<InventoryState>,
    mut query: Query<(&InventorySlot, &Interaction, &mut BorderColor), (Changed<Interaction>, With<Button>)>,
) {
    for (slot, interaction, mut border) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                *border = BorderColor::all(theme.border_hover);
            }

            Interaction::None => {
                let selected = slot.kind == SlotKind::Hotbar && slot.index == state.hotbar.active_index;

                *border = BorderColor::all(
                    if selected {
                        theme.border_selected
                    } else {
                        theme.border_default
                    }
                );
            }

            Interaction::Pressed => {
                *border = BorderColor::all(
                    theme.border_selected
                );
            }
        }
    }
}


/// 插槽点击逻辑路由器
pub fn handle_slot_clicked_system(
    mut reader: MessageReader<SlotClickedEvent>,
    mut inventory: ResMut<InventoryState>,
) {
    for event in reader.read() {
        match event.kind {
            // 创造模式
            SlotKind::CreativeGrid => {
                let Some(item) = inventory.creative.visible_items.get(event.index).cloned()
                else {
                    continue;
                };

                let hotbar_index = inventory.hotbar.active_index;

                inventory.hotbar.items[hotbar_index] = item.clone();
                inventory.add_recent(item);
            }

            // 最近使用
            SlotKind::Recent => {
                let Some(item) = inventory.recent.items.get(event.index).cloned()
                else {
                    continue;
                };

                let hotbar_index = inventory.hotbar.active_index;

                inventory.hotbar.items[hotbar_index] = item.clone();
                inventory.add_recent(item);
            }

            // 快捷栏
            SlotKind::Hotbar => {
                inventory.hotbar.active_index = event.index;
            }

            // 未来扩展
            SlotKind::Container => {
            }
            SlotKind::SurvivalBackpack => {
            }
        }
    }
}

/// 类别交互系统
pub fn category_interaction_system(
    mut reader: MessageReader<CategoryClickedEvent>,
    mut inventory: ResMut<InventoryState>,
) {
    for event in reader.read() {
        inventory.creative.selected_tab = event.category_index;
    }
}

/// 激活搜索框系统
pub fn activate_search_box_system(
    mut click_events: MessageReader<Pointer<Click>>,
    query: Query<(), With<CreativeSearchInput>>,
    mut search_state: ResMut<SearchInputState>,
) {
    for ev in click_events.read() {
        if query.get(ev.entity).is_ok() {
            search_state.active = true;
        }
    }
}

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
        let Some(text) = &ev.text else { continue; };

        for ch in text.chars() {
            if !ch.is_control() {
                inventory.creative.search_text.push(ch);
            }
        }
    }
}
/// 搜索退出系统
pub fn search_escape_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut search_state: ResMut<SearchInputState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        search_state.active = false;
    }
}

/// 更新搜索文本
pub fn update_search_text_display_system(
    inventory: Res<InventoryState>,
    query: Query<&Children, With<CreativeSearchBox>>,
    mut text_query: Query<&mut Text>,
) {
    let Ok(children) = query.single() else {
        return;
    };

    let search = inventory.creative.search_text.clone();

    for child in children.iter() {
        if let Ok(mut text) = text_query.get_mut(child) {
            if search.is_empty() {
                *text = Text::new("🔍 搜索...");
            } else {
                *text = Text::new(search.clone());
            }
        }
    }
}

/// 类别标签交互系统
pub fn category_tab_interaction_system(
    mut interaction_query: Query<(&Interaction, &CategoryTab), (Changed<Interaction>, With<Button>)>,
    mut inventory: ResMut<InventoryState>,
) {
    for (interaction, tab) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        inventory.creative.selected_tab = tab.category_index;
    }
}

/// 搜索框交互系统 
pub fn search_box_interaction_system(
    mut query: Query<(&Interaction, Entity), (Changed<Interaction>, With<CreativeSearchInput>)>,
    mut search_state: ResMut<SearchInputState>,
) {
    for (interaction, _) in &mut query {
        if *interaction == Interaction::Pressed {
            search_state.active = true;

            info!("Search Activated");
        }
    }
}