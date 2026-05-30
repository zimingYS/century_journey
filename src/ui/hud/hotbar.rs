use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use crate::ui::components::{HudHotbarContainer, HudHotbarSlot};
use crate::ui::resources::inventory_ui_state::InventoryUiState;
use crate::voxel::registry::BlockRegistry;
use crate::voxel::types::VoxelType;

pub fn spawn_hotbar_ui_system(mut commands: Commands){
    commands.spawn((
        HudHotbarContainer,
        Node {
            position_type: PositionType::Absolute,
            // 左右居中平衡
            left: Val::Percent(35.0),
            bottom: Val::Px(20.0),
            width: Val::Percent(30.0),
            height: Val::Px(60.0),
            // 网格排列
            display: Display::Grid,
            // 平分 9 格
            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(5.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
    )).with_children(|parent| {
        for index in 0..9 {
            parent.spawn((
                HudHotbarSlot { index },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::srgb(0.05, 0.05, 0.05)),
            ));
        }
    });
}

pub fn update_hotbar_ui_system(
    registry: Option<Res<BlockRegistry>>,
    inventory_ui_state: Res<InventoryUiState>,
    mut slot_query: Query<(&HudHotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
) {
    let Some(reg) = registry else { return; };

    for (slot, mut bg_color, mut border_color) in &mut slot_query {
        let identifier = &inventory_ui_state.hotbar_items[slot.index];

        // 刷新格子内的方块颜色
        if identifier == "century_journey:air" {
            // 空气显示半透明阴影
            *bg_color = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3));
        } else if let Some(prop) = reg.id_to_properties.values().find(|p| &p.identifier == identifier) {
            // 临时颜色代替（暂定绿色，以后加了UI贴图可以直接在这里换成渲染图标）
            *bg_color = BackgroundColor(Color::srgb(0.5, 0.7, 0.5));
        } else {
            // 未知方块降级为灰色兜底
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
        }

        // 刷新高亮框
        if slot.index == inventory_ui_state.active_hotbar_index {
            // 选中格子的边框变成白色凸显
            *border_color = BorderColor::all(Color::srgb(1.0, 1.0, 1.0));
        } else {
            // 未选中的格子恢复暗色
            *border_color = BorderColor::all(Color::srgb(0.05, 0.05, 0.05));
        }
    }
}

pub fn handle_hotbar_switch_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut inventory_ui_state: ResMut<InventoryUiState>,
) {
    // 背包打开时禁止用滚轮和数字键切格子
    if inventory_ui_state.is_inventory_open { return; }

    // 键盘数字键处理
    let num_keys = [
        (KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3), (KeyCode::Digit5, 4), (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6), (KeyCode::Digit8, 7), (KeyCode::Digit9, 8),
    ];

    for (key, idx) in num_keys {
        if keyboard.just_pressed(key) {
            inventory_ui_state.active_hotbar_index = idx;
            return;
        }
    }

    // 鼠标滚轮处理
    for event in mouse_wheel_events.read() {
        let current = inventory_ui_state.active_hotbar_index as i32;
        // 滚轮向上往左移，滚轮向下往右移
        let mut next = if event.y > 0.0 { current - 1 } else { current + 1 };

        if next < 0 { next = 8; }
        if next > 8 { next = 0; }

        inventory_ui_state.active_hotbar_index = next as usize;
    }
}