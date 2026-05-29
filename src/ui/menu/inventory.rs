use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use crate::core::constant::TOTAL_SLOTS;
use crate::ui::components::{CreativeInventoryMenu, PaletteSlot};
use crate::ui::resources::inventory_ui_state::InventoryUiState;
use crate::voxel::types::VoxelType;

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
    // 外部背景
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
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        // 半透明黑灰色背景
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
        BorderColor::all(Color::srgb(0.05, 0.05, 0.05)),
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("Creative Inventory"),
            TextFont { font_size: FontSize::Px(20.0), ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() }
        ));

        // 物品网格
        parent.spawn(Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
            grid_auto_rows: GridTrack::px(50.0),
            row_gap: Val::Px(4.0),
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|grid| {
            // 生成格子
            for index in 0..TOTAL_SLOTS {
                // 如果有对应的方块就取出来没有就用空气填充
                let voxel = inventory_ui_state
                    .creative_palette
                    .get(index)
                    .copied()
                    .unwrap_or(VoxelType::Air);

                grid.spawn((
                    PaletteSlot { voxel_type: voxel },
                    // 让节点能够响应点击
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
                    BorderColor::all(Color::srgb(0.15, 0.15, 0.15)),
                )).with_children(|slot_node| {
                    let bg_color = if voxel == VoxelType::Air {
                        // 空气显示阴影空格
                        Color::srgba(0.0, 0.0, 0.0, 0.3)
                    } else {
                        // 渲染方块颜色
                        voxel.get_voxel_color()
                    };

                    slot_node.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(bg_color),
                    ));
                });
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
            if slot.voxel_type == VoxelType::Air { continue; }
            
            let current_slot = inventory_ui_state.active_hotbar_index;
            // 将选中的方块类型塞进当前手持的快捷栏格子中
            inventory_ui_state.hotbar_items[current_slot] = slot.voxel_type;

            println!("已将持有的格子修改为 -> {:?}", slot.voxel_type);
        }
    }
}

/// 物品槽悬浮高亮反馈
pub fn palette_slot_visual_system(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, &PaletteSlot),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut border_color, _slot) in &mut interaction_query {
        match *interaction {
            // 鼠标悬浮在格子上,边框变成亮白色
            Interaction::Hovered => {
                *border_color = BorderColor::all(Color::srgb(0.9, 0.9, 0.2));
            }
            // 鼠标按下格子,边框变成纯白色
            Interaction::Pressed => {
                *border_color = BorderColor::all(Color::srgb(1.0, 1.0, 1.0));
            }
            // 鼠标移开,恢复暗色内凹边框
            Interaction::None => {
                *border_color = BorderColor::all(Color::srgb(0.1, 0.1, 0.1));
            }
        }
    }
}
