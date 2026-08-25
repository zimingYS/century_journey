//! 构建生存物品栏的静态布局和槽位区域。

use bevy::prelude::*;

use super::preview::spawn_player_preview;
use crate::client::ui::components::{
    CompactBackpackButton, SortBackpackButton, SurvivalAccessoryPanel, SurvivalDefenseText,
    SurvivalEquipmentPanel, SurvivalHealthText, SurvivalHotbarPanel, SurvivalHungerText,
    SurvivalInventoryOverlay, SurvivalInventoryRoot, SurvivalItemGrid,
};
use crate::client::ui::localization::LocalizedText;
use crate::client::ui::navigation::{UiScreenAudience, UiScreenRoot};
use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::screens::crafting::CraftingHost;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControlKind, spawn_text_button};
use crate::client::ui::widgets::slot::{SlotKind, spawn_empty_slot_with_placeholder};
use crate::engine::localization::Localization;
use crate::game::inventory::state::{AccessorySlotDefinitions, EquipmentSlot};

const SURVIVAL_PANEL_WIDTH: f32 = 708.0;
const SURVIVAL_PANEL_HEIGHT: f32 = 680.0;
/// 生存背包主区域槽位的固定像素尺寸。
pub(super) const MAIN_SLOT_SIZE: f32 = 54.0;
const SIDE_SLOT_SIZE: f32 = 40.0;
const SIDE_PANEL_WIDTH: f32 = 58.0;
/// 生存物品栏位于 HUD 之上，避免被快捷栏和状态条遮挡。
const SURVIVAL_OVERLAY_Z: i32 = 1000;
/// 创建生存物品栏的稳定 UI 实体层级。
/// 背包根界面一次性构建多个相互关联区域，启动资源保持显式传入。
#[allow(clippy::too_many_arguments)]
pub fn spawn_survival_inventory_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
    accessory_definitions: Res<AccessorySlotDefinitions>,
    localization: Res<Localization>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // 所有玩家共用 player.glb，不再需要 ProgramModeConfig / StandardMaterial 资源；
    // 这里保留 ui_font / theme / accessory_definitions 等回调上下文参数，
    // 调用方未来如果要按风格切换预览可以接回去。
    let _ = (&theme, &ui_font, &accessory_definitions);
    let preview_image =
        spawn_player_preview(&mut commands, &asset_server, &mut images, &mut meshes);

    commands
        .spawn((
            SurvivalInventoryOverlay,
            UiScreenRoot::inventory(UiScreenAudience::Survival),
            Name::new("SurvivalOverlay"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ZIndex(SURVIVAL_OVERLAY_Z),
            BackgroundColor(Color::srgba(0.015, 0.02, 0.025, 0.78)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    SurvivalInventoryRoot,
                    UiFrameKind::Survival,
                    Name::new("SurvivalRoot"),
                    Node {
                        width: Val::Px(SURVIVAL_PANEL_WIDTH),
                        height: Val::Px(SURVIVAL_PANEL_HEIGHT),
                        max_width: Val::Percent(100.0),
                        max_height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.075, 0.075, 0.085, 0.98)),
                    BorderColor::all(Color::srgba(0.38, 0.38, 0.42, 1.0)),
                ))
                .with_children(|root| {
                    root.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(354.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|top| {
                        build_equipment_panel(top, &theme, &ui_font, &localization);
                        build_preview_panel(
                            top,
                            preview_image.clone(),
                            &theme,
                            &ui_font,
                            &localization,
                        );
                        build_accessory_panel(
                            top,
                            &accessory_definitions,
                            &theme,
                            &ui_font,
                            &localization,
                        );
                    });

                    build_backpack_panel(root, &theme, &ui_font, &localization);
                    build_survival_hotbar_panel(root, &theme);
                });
        });
}

fn build_equipment_panel(
    parent: &mut ChildSpawnerCommands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    let side_theme = slot_theme(theme, SIDE_SLOT_SIZE);
    parent
        .spawn((
            SurvivalEquipmentPanel,
            Node {
                width: Val::Px(SIDE_PANEL_WIDTH),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.bg_sidebar),
            BorderColor::all(Color::srgba(0.1, 0.76, 0.7, 0.8)),
        ))
        .with_children(|panel| {
            spawn_heading(panel, "survival.equipment", theme, ui_font, localization);
            for (index, equipment_slot) in EquipmentSlot::ALL.into_iter().enumerate() {
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(SIDE_SLOT_SIZE),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(9.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_empty_slot_with_placeholder(
                            row,
                            SlotKind::SurvivalEquipment,
                            index,
                            equipment_slot.placeholder_key(),
                            &side_theme,
                            ui_font,
                            localization,
                        );
                    });
            }
        });
}

fn build_accessory_panel(
    parent: &mut ChildSpawnerCommands,
    definitions: &AccessorySlotDefinitions,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    let side_theme = slot_theme(theme, SIDE_SLOT_SIZE);
    parent
        .spawn((
            SurvivalAccessoryPanel,
            Node {
                width: Val::Px(SIDE_PANEL_WIDTH),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(5.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.bg_sidebar),
            BorderColor::all(Color::srgba(0.92, 0.7, 0.08, 0.9)),
        ))
        .with_children(|panel| {
            spawn_heading(panel, "survival.accessories", theme, ui_font, localization);
            for (index, definition) in definitions.slots.iter().enumerate() {
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(SIDE_SLOT_SIZE),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(9.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_empty_slot_with_placeholder(
                            row,
                            SlotKind::SurvivalAccessory,
                            index,
                            &definition.placeholder_key,
                            &side_theme,
                            ui_font,
                            localization,
                        );
                    });
            }
        });
}

fn build_preview_panel(
    parent: &mut ChildSpawnerCommands,
    preview_image: Handle<Image>,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    parent
        .spawn((
            CraftingHost,
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|center| {
            center
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        height: Val::Percent(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.13, 0.15, 1.0)),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|preview| {
                    preview.spawn((
                        ImageNode {
                            image: preview_image,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                });

            center
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(42.0),
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.bg_content),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|stats| {
                    spawn_stat_text::<SurvivalHealthText>(
                        stats,
                        "survival.health-placeholder",
                        Color::srgb(0.96, 0.25, 0.24),
                        ui_font,
                        localization,
                    );
                    spawn_stat_text::<SurvivalDefenseText>(
                        stats,
                        "survival.defense-placeholder",
                        Color::srgb(0.55, 0.7, 0.82),
                        ui_font,
                        localization,
                    );
                    spawn_stat_text::<SurvivalHungerText>(
                        stats,
                        "survival.hunger-placeholder",
                        Color::srgb(0.88, 0.55, 0.25),
                        ui_font,
                        localization,
                    );
                });

            center
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(42.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(7.0),
                        padding: UiRect::horizontal(Val::Px(7.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.bg_content),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|actions| {
                    spawn_action_button::<CompactBackpackButton>(
                        actions,
                        "survival.compact",
                        theme,
                        ui_font,
                        localization,
                    );
                    spawn_action_button::<SortBackpackButton>(
                        actions,
                        "survival.sort",
                        theme,
                        ui_font,
                        localization,
                    );
                });
        });
}

fn build_backpack_panel(
    root: &mut ChildSpawnerCommands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    let grid_width = MAIN_SLOT_SIZE * 9.0 + theme.slot_gap * 8.0 + 12.0;
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(190.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(5.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.055, 0.055, 0.065, 1.0)),
    ))
    .with_children(|section| {
        section
            .spawn(Node {
                width: Val::Px(grid_width),
                height: Val::Px(19.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|title| {
                spawn_label(title, "survival.backpack", theme, ui_font, localization)
            });
        section.spawn((
            SurvivalItemGrid,
            Name::new("SurvivalGrid"),
            Node {
                width: Val::Px(grid_width),
                height: Val::Px(MAIN_SLOT_SIZE * 3.0 + theme.slot_gap * 2.0 + 12.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(9, MAIN_SLOT_SIZE),
                grid_template_rows: RepeatedGridTrack::px(3, MAIN_SLOT_SIZE),
                column_gap: Val::Px(theme.slot_gap),
                row_gap: Val::Px(theme.slot_gap),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.bg_content),
            BorderColor::all(theme.border_default),
        ));
    });
}

fn build_survival_hotbar_panel(root: &mut ChildSpawnerCommands, theme: &UiTheme) {
    let width = MAIN_SLOT_SIZE * 9.0 + theme.slot_gap * 8.0 + 12.0;
    root.spawn((
        SurvivalHotbarPanel,
        Node {
            width: Val::Px(width),
            height: Val::Px(66.0),
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(5.0)),
            column_gap: Val::Px(theme.slot_gap),
            ..default()
        },
        BackgroundColor(theme.bg_content),
        BorderColor::all(Color::srgba(0.4, 0.78, 0.25, 0.85)),
    ));
}

fn spawn_heading(
    parent: &mut ChildSpawnerCommands,
    key: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    parent.spawn((
        LocalizedText::new(key),
        Text::new(localization.get(key)),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(17.0),
            ..default()
        },
        TextColor(theme.text_primary),
        Node {
            height: Val::Px(22.0),
            ..default()
        },
    ));
}

fn spawn_label(
    parent: &mut ChildSpawnerCommands,
    key: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    parent.spawn((
        LocalizedText::new(key),
        Text::new(localization.get(key)),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(theme.body_font_size),
            ..default()
        },
        TextColor(theme.text_secondary),
    ));
}

/// 生成状态条初始文本；实际数值由同步系统每帧重写，不使用本地化标记。
fn spawn_stat_text<M: Component + Default>(
    parent: &mut ChildSpawnerCommands,
    key: &str,
    color: Color,
    ui_font: &UiFont,
    localization: &Localization,
) {
    parent.spawn((
        M::default(),
        Text::new(localization.get(key)),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(color),
    ));
}

fn spawn_action_button<M: Component + Default>(
    parent: &mut ChildSpawnerCommands,
    label_key: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    let entity = spawn_text_button(
        parent,
        M::default(),
        localization.get(label_key),
        UiControlKind::Button,
        theme,
        ui_font,
    );
    parent
        .commands()
        .entity(entity)
        .insert(LocalizedText::new(label_key))
        .insert(Node {
            width: Val::Px(72.0),
            height: Val::Px(29.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        });
}

/// 根据槽位尺寸返回生存背包使用的局部视觉主题。
pub(super) fn slot_theme(theme: &UiTheme, size: f32) -> UiTheme {
    let mut result = theme.clone();
    result.slot_size = size;
    result.slot_border = 1.0;
    result
}
