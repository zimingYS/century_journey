//! 把鼠标和键盘界面操作转换为 Game 层背包与合成命令。

use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SearchInputState,
    SlotInteractionEvent, SlotKind,
};
use crate::game::crafting::grid::ActiveCrafting;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::{InventoryState, LocalInventory, LocalInventoryMut};
use crate::game::player::identity::{LocalPlayer, PlayerId};
use crate::shared::states::{InputContext, InputContextState};
use bevy::input::mouse::MouseWheel;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::EditableText;
use std::collections::HashSet;

/// 客户端槽位拖拽过程中的起始槽位和指针状态。
#[derive(Resource, Default)]
pub struct SlotDragState {
    button: Option<MouseButton>,
    visited: HashSet<(SlotKind, usize)>,
}

/// 槽位左键或 Shift 左键交互。
/// 查询过滤器只处理本帧变化的可点击槽位，保持显式可避免误消费其他按钮。
#[allow(clippy::type_complexity)]
pub fn slot_interaction_system(
    query: Query<(&Interaction, &InventorySlot), (Changed<Interaction>, With<Button>)>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SlotInteractionEvent>,
    context: Single<(&PlayerId, &ActiveCrafting), With<LocalPlayer>>,
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
        writer.write(slot_interaction_event(&context, slot, action));
    }
}

/// 槽位右键交互。
pub fn slot_right_click_system(
    query: Query<(&Interaction, &InventorySlot), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SlotInteractionEvent>,
    context: Single<(&PlayerId, &ActiveCrafting), With<LocalPlayer>>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
        return;
    }

    for (interaction, slot) in &query {
        if *interaction == Interaction::Hovered {
            writer.write(slot_interaction_event(
                &context,
                slot,
                SlotAction::RightClick,
            ));
            break;
        }
    }
}

/// Mouse Tweaks 风格的槽位拖拽：右键逐个放置，左键连续移动，Shift 左键连续快移。
pub fn slot_drag_interaction_system(
    query: Query<(&Interaction, &InventorySlot), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut drag: ResMut<SlotDragState>,
    mut writer: MessageWriter<SlotInteractionEvent>,
    context: Single<(&PlayerId, &ActiveCrafting), With<LocalPlayer>>,
) {
    let active_button = if mouse.pressed(MouseButton::Right) {
        Some(MouseButton::Right)
    } else if mouse.pressed(MouseButton::Left) {
        Some(MouseButton::Left)
    } else {
        None
    };

    if drag.button != active_button {
        drag.button = active_button;
        drag.visited.clear();
    }
    let Some(button) = active_button else {
        return;
    };

    let just_started = mouse.just_pressed(button);
    for (interaction, slot) in &query {
        if !matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
            continue;
        }
        if !drag.visited.insert((slot.kind, slot.index)) || just_started {
            continue;
        }
        writer.write(slot_interaction_event(
            &context,
            slot,
            drag_action(button, shift_pressed(&keyboard)),
        ));
    }
}

fn shift_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
}

fn drag_action(button: MouseButton, shift: bool) -> SlotAction {
    match (button, shift) {
        (MouseButton::Left, true) => SlotAction::ShiftClick,
        (MouseButton::Right, _) => SlotAction::RightClick,
        _ => SlotAction::LeftClick,
    }
}

/// 将悬停槽位上的滚轮操作转换为单个物品转移命令。
pub fn slot_wheel_interaction_system(
    query: Query<(&Interaction, &InventorySlot), With<Button>>,
    mut wheel: MessageReader<MouseWheel>,
    mut writer: MessageWriter<SlotInteractionEvent>,
    context: Single<(&PlayerId, &ActiveCrafting), With<LocalPlayer>>,
) {
    let hovered = query
        .iter()
        .find(|(interaction, _)| {
            matches!(**interaction, Interaction::Hovered | Interaction::Pressed)
        })
        .map(|(_, slot)| *slot);
    let Some(slot) = hovered else {
        wheel.clear();
        return;
    };

    for event in wheel.read() {
        let action = if event.y > 0.0 {
            SlotAction::ScrollUp
        } else if event.y < 0.0 {
            SlotAction::ScrollDown
        } else {
            continue;
        };
        let steps = (event.y.abs().ceil() as usize).clamp(1, 8);
        for _ in 0..steps {
            writer.write(slot_interaction_event(&context, &slot, action));
        }
    }
}

/// 背包打开时，悬停槽位并按 Q 丢弃物品。
pub fn slot_q_drop_system(
    query: Query<(&Interaction, &InventorySlot), With<Button>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SlotInteractionEvent>,
    context: Single<(&PlayerId, &ActiveCrafting), With<LocalPlayer>>,
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
            writer.write(slot_interaction_event(&context, slot, action));
            break;
        }
    }
}

fn slot_interaction_event(
    context: &Single<(&PlayerId, &ActiveCrafting), With<LocalPlayer>>,
    slot: &InventorySlot,
    action: SlotAction,
) -> SlotInteractionEvent {
    let (player_id, active) = **context;
    let container_id = match slot.kind {
        SlotKind::Container(crate::game::inventory::container::ContainerKind::PlayerCrafting) => {
            None
        }
        SlotKind::Container(_) => active.container_id,
        _ => None,
    };
    SlotInteractionEvent {
        player_id: *player_id,
        container_id,
        kind: slot.kind,
        index: slot.index,
        action,
    }
}

/// 分类标签点击交互。
/// 查询过滤器限定创造分类标签，不能与普通主题按钮共享处理。
#[allow(clippy::type_complexity)]
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
    inventory: LocalInventory,
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
    mut inventory: LocalInventoryMut,
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

/// 分类切换事件处理。
pub fn handle_category_clicked_system(
    mut reader: MessageReader<CategoryClickedEvent>,
    mut inventory: LocalInventoryMut,
) {
    for event in reader.read() {
        inventory.creative.selected_tab = event.category_index;
    }
}

/// 槽位边框高亮。
/// 同一查询同时更新悬停槽位边框，过滤器用于隔离其他按钮组件。
#[allow(clippy::type_complexity)]
pub fn slot_hover_system(
    theme: Res<UiTheme>,
    state: LocalInventory,
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

/// 组装槽位输入采集、分类切换和搜索框同步系统。
pub struct UiInteractionPlugin;

impl Plugin for UiInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SlotDragState>()
            .add_systems(
                Update,
                (
                    slot_interaction_system,
                    slot_right_click_system,
                    slot_drag_interaction_system,
                    slot_wheel_interaction_system,
                    slot_q_drop_system,
                    category_tab_interaction_system,
                )
                    .run_if(|context: Res<InputContextState>| {
                        context.active() == InputContext::Inventory
                    }),
            )
            .add_systems(
                Update,
                handle_category_clicked_system.run_if(local_inventory_open),
            )
            .add_systems(
                Update,
                (
                    sync_search_input_focus_system,
                    sync_search_text_from_editable_system,
                )
                    .chain(),
            );
    }
}

fn local_inventory_open(query: Query<&InventoryState, With<LocalPlayer>>) -> bool {
    query.single().is_ok_and(|inventory| inventory.opened)
}
#[cfg(test)]
#[path = "../../../tests/unit/client/ui/interaction.rs"]
mod tests;
