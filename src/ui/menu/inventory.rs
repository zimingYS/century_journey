use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use crate::ui::components::{CreativeInventoryMenu, PaletteSlot};
use crate::ui::resources::inventory_ui_state::InventoryUiState;

/// E键打开物品栏
pub fn toggle_inventory_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory_ui_state: ResMut<InventoryUiState>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    menu_query: Query<Entity, With<CreativeInventoryMenu>>,
){
    if !keyboard.just_pressed(KeyCode::KeyE) { return; }

    inventory_ui_state.is_inventory_open = !inventory_ui_state.is_inventory_open;

    let Ok(mut cursor_options) = cursor_options_query.single_mut() else { return; };

    if inventory_ui_state.is_inventory_open {
        // 打开背包物品栏
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;

        // 渲染创造模式物品栏
        spawn_creative_menu_ui(&mut commands, &inventory_ui_state);
    }else {
        // 关闭背包物品栏
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;

        for entity in &menu_query {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_creative_menu_ui(commands: &mut Commands, inventory_ui_state: &InventoryUiState) {
    commands.spawn((
        CreativeInventoryMenu,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(25.0),
            width: Val::Percent(40.0),
            height: Val::Percent(50.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)), // 半透明黑灰色背景
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("Creative Inventory"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() }
        ));

        // 物品网格
        parent.spawn(Node {
            display: Display::Grid,
            // 横向9格
            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
            // 格子高度
            grid_auto_rows: GridTrack::px(50.0),
            // 格子间距
            row_gap: Val::Px(6.0),
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|grid| {
            // 循环遍历调色板里的方块，生成格子
            for voxel in &inventory_ui_state.creative_palette {
                grid.spawn((
                    PaletteSlot { voxel_type: *voxel },
                    Button, // 让节点能够响应点击
                    Node {
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    // 格子背景色暂时用方块自带的颜色
                    BackgroundColor(voxel.get_voxel_color()),
                ));
            }
        });
    });
}

pub fn palette_click_system(
    mut inventory_ui_state: ResMut<InventoryUiState>,
    interaction_query: Query<(&Interaction, &PaletteSlot), (Changed<Interaction>, With<Button>)>,
) {
    if !inventory_ui_state.is_inventory_open { return; }

    for (interaction, slot) in &interaction_query {
        if *interaction == Interaction::Pressed {
            let current_slot = inventory_ui_state.active_hotbar_index;
            // 将选中的方块类型塞进当前手持的快捷栏格子中
            inventory_ui_state.hotbar_items[current_slot] = slot.voxel_type;

            println!("已将持有的格子修改为 -> {:?}", slot.voxel_type);
        }
    }
}
