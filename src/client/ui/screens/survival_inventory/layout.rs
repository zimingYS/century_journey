//! 构建生存物品栏的静态布局，基于像素级设计稿绝对定位。
//!
//! 设计稿尺寸 1444 × 856，所有子元素使用绝对定位
//! 精确对齐底图上的槽位、预览区、按钮等视觉元素。

use bevy::prelude::*;

use super::preview::spawn_player_preview;
use crate::client::ui::components::{
    SurvivalCloseButton, SurvivalEquipmentPanel, SurvivalHotbarPanel, SurvivalInventoryOverlay,
    SurvivalInventoryRoot, SurvivalItemGrid, SurvivalOffhandSlot, SurvivalTitleText,
};
use crate::client::ui::localization::LocalizedText;
use crate::client::ui::navigation::{UiScreenAudience, UiScreenRoot};
use crate::client::ui::resources::survival_assets::SurvivalUiAssets;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{SlotKind, spawn_empty_slot};
use crate::engine::localization::Localization;
use crate::game::inventory::container::ContainerKind;
use crate::game::inventory::state::{AccessorySlotDefinitions, EquipmentSlot};

// ===== 设计稿基准尺寸 =====
/// 生存背包面板设计稿宽度（像素）。
pub const SURVIVAL_PANEL_WIDTH: f32 = 1444.0;
/// 生存背包面板设计稿高度（像素）。
pub const SURVIVAL_PANEL_HEIGHT: f32 = 856.0;
/// 面板整体显示缩放：设计稿按此比例渲染，内部布局仍用设计稿像素坐标。
///
/// 通过根节点的 [`UiTransform`] 以面板中心为锚点缩放，
/// 子节点渲染与指针拾取都会遵循该变换。
pub const SURVIVAL_PANEL_SCALE: f32 = 0.7;

// ===== 主背包网格（4 行 × 9 列） =====
/// 主背包槽位尺寸（设计稿像素）。
pub const BACKPACK_SLOT_SIZE: f32 = 91.0;
/// 主背包槽位水平间距（设计稿像素，列间距 10 → 步距 101）。
const BACKPACK_COLUMN_GAP: f32 = 10.0;
/// 主背包槽位垂直间距（设计稿像素，行间距 11 → 步距 102）。
const BACKPACK_ROW_GAP: f32 = 11.0;
/// 主背包列数。
pub const BACKPACK_COLUMNS: usize = 9;
/// 主背包行数。
pub const BACKPACK_ROWS: usize = 4;
/// 主背包网格左上角（设计稿像素）。
const BACKPACK_GRID_ORIGIN: Vec2 = Vec2::new(488.0, 133.0);

// ===== 快捷栏（1 行 × 9 列） =====
/// 快捷栏槽位宽度（设计稿像素）。
pub const HOTBAR_SLOT_WIDTH: f32 = 82.0;
/// 快捷栏槽位高度（设计稿像素）。
pub const HOTBAR_SLOT_HEIGHT: f32 = 83.0;
/// 快捷栏槽位间距（设计稿实测平均步距 100.375）。
const HOTBAR_SLOT_GAP: f32 = 18.375;
/// 快捷栏左上角（设计稿像素）。
const HOTBAR_ORIGIN: Vec2 = Vec2::new(492.0, 707.0);
/// 选中框宽度（素材 hotbar_selection.png）。
pub const HOTBAR_SELECTION_WIDTH: f32 = 106.0;
/// 选中框高度（素材 hotbar_selection.png）。
pub const HOTBAR_SELECTION_HEIGHT: f32 = 111.0;
/// 选中框相对槽位左上角的偏移（设计稿模板匹配结果）。
pub const HOTBAR_SELECTION_OFFSET: Vec2 = Vec2::new(-15.0, -15.0);

// ===== 左侧装备槽（4 个垂直） =====
/// 装备槽位宽度（设计稿框线 x 65-153，含边框整体宽 89）。
const EQUIP_SLOT_WIDTH: f32 = 89.0;
/// 装备槽位高度。
const EQUIP_SLOT_HEIGHT: f32 = 73.0;
/// 装备槽垂直间距。
const EQUIP_SLOT_GAP: f32 = 23.0;
/// 第一个装备槽（头盔）左上角。
const EQUIP_ORIGIN: Vec2 = Vec2::new(65.0, 139.0);
/// 装备槽数量（头盔、胸甲、护腿、靴子）。
const EQUIP_SLOT_COUNT: usize = 4;

// ===== 副手/盾牌槽（角色预览右下角） =====
/// 盾牌槽尺寸（槽位框内部图标区为 64×64 正方形）。
const OFFHAND_SLOT_SIZE: f32 = 64.0;
/// 盾牌槽左上角（设计稿像素，框线 x 370-451 / y 458-541 的内部区域）。
const OFFHAND_ORIGIN: Vec2 = Vec2::new(380.0, 467.0);

// ===== 合成区（左下角 2×2 + 输出，槽位框与箭头已绘制在底图中） =====
/// 合成输入槽尺寸。
const CRAFT_INPUT_SIZE: f32 = 91.0;
/// 合成输入槽列间距（列步距 101）。
const CRAFT_INPUT_COLUMN_GAP: f32 = 10.0;
/// 合成输入槽行间距（行步距 102）。
const CRAFT_INPUT_ROW_GAP: f32 = 11.0;
/// 合成 2×2 网格左上角。
const CRAFT_GRID_ORIGIN: Vec2 = Vec2::new(77.0, 583.0);
/// 合成输出槽左上角。
const CRAFT_OUTPUT_ORIGIN: Vec2 = Vec2::new(343.0, 634.0);
/// 合成输出槽尺寸。
const CRAFT_OUTPUT_SIZE: f32 = 91.0;

// ===== 角色预览区 =====
/// 预览区左上角。
const PREVIEW_ORIGIN: Vec2 = Vec2::new(169.0, 128.0);
/// 预览区尺寸。
const PREVIEW_SIZE: Vec2 = Vec2::new(272.0, 410.0);

// ===== 标题栏 =====
/// 标题文字位置（左上角）。
const TITLE_ORIGIN: Vec2 = Vec2::new(120.0, 40.0);

// ===== 关闭按钮 =====
/// 关闭按钮尺寸。
const CLOSE_BUTTON_SIZE: f32 = 52.0;
/// 关闭按钮左上角。
const CLOSE_BUTTON_ORIGIN: Vec2 = Vec2::new(1368.0, 28.0);

/// 生存物品栏遮罩层 Z 索引。
const SURVIVAL_OVERLAY_Z: i32 = 1000;

/// 创建生存物品栏的静态布局。
///
/// 设计稿整图作为背景，子节点全部使用 `PositionType::Absolute`
/// 按设计稿像素坐标精确定位。
#[allow(clippy::too_many_arguments)]
pub fn spawn_survival_inventory_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
    accessory_definitions: Res<AccessorySlotDefinitions>,
    localization: Res<Localization>,
    survival_assets: Res<SurvivalUiAssets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let _ = (&theme, &accessory_definitions);
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
                    Name::new("SurvivalRoot"),
                    Node {
                        width: Val::Px(SURVIVAL_PANEL_WIDTH),
                        height: Val::Px(SURVIVAL_PANEL_HEIGHT),
                        position_type: PositionType::Relative,
                        max_width: Val::Percent(100.0),
                        max_height: Val::Percent(100.0),
                        ..default()
                    },
                    // 设计稿整图作为背景，不切九宫格。
                    ImageNode {
                        image: survival_assets.panel.clone(),
                        ..default()
                    },
                    // 以面板中心为锚点整体缩放：布局坐标保持设计稿像素，
                    // 渲染与拾取都按缩放后的位置生效。
                    UiTransform::from_scale(Vec2::splat(SURVIVAL_PANEL_SCALE)),
                ))
                .with_children(|root| {
                    // 标题文字
                    spawn_title_text(root, &ui_font, &localization);

                    // 关闭按钮
                    spawn_close_button(root);

                    // 玩家模型预览
                    spawn_player_preview_area(root, preview_image.clone());

                    // 左侧装备槽（4 个：头盔、胸甲、护腿、靴子）
                    spawn_equipment_slots(root, &theme, &ui_font);

                    // 副手/盾牌槽
                    spawn_offhand_slot(root, &theme, &ui_font);

                    // 合成区（2×2 输入格 + 输出格，槽位框已绘制在底图）
                    spawn_crafting_slots(root, &theme, &ui_font);

                    // 主背包网格（4 行 × 9 列）
                    spawn_backpack_grid(root);

                    // 快捷栏（9 格）
                    spawn_hotbar_panel(root);
                });
        });
}

fn spawn_title_text(
    parent: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    localization: &Localization,
) {
    parent.spawn((
        SurvivalTitleText,
        LocalizedText::new("survival.title"),
        Text::new(localization.get("survival.title")),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.82, 0.65)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(TITLE_ORIGIN.x),
            top: Val::Px(TITLE_ORIGIN.y),
            ..default()
        },
    ));
}

fn spawn_close_button(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        SurvivalCloseButton,
        Button,
        Pickable::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(CLOSE_BUTTON_ORIGIN.x),
            top: Val::Px(CLOSE_BUTTON_ORIGIN.y),
            width: Val::Px(CLOSE_BUTTON_SIZE),
            height: Val::Px(CLOSE_BUTTON_SIZE),
            ..default()
        },
        // 关闭按钮使用整图（红色 X 图标在底图上已有，这里放透明按钮用于交互）
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.001)),
    ));
}

fn spawn_player_preview_area(parent: &mut ChildSpawnerCommands, preview_image: Handle<Image>) {
    parent
        .spawn((
            Name::new("PreviewArea"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PREVIEW_ORIGIN.x),
                top: Val::Px(PREVIEW_ORIGIN.y),
                width: Val::Px(PREVIEW_SIZE.x),
                height: Val::Px(PREVIEW_SIZE.y),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
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
}

fn spawn_equipment_slots(parent: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    parent
        .spawn((
            SurvivalEquipmentPanel,
            Name::new("EquipmentSlots"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(EQUIP_ORIGIN.x),
                top: Val::Px(EQUIP_ORIGIN.y),
                width: Val::Px(EQUIP_SLOT_WIDTH),
                height: Val::Px(
                    EQUIP_SLOT_HEIGHT * EQUIP_SLOT_COUNT as f32
                        + EQUIP_SLOT_GAP * (EQUIP_SLOT_COUNT - 1) as f32,
                ),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(EQUIP_SLOT_GAP),
                ..default()
            },
        ))
        .with_children(|panel| {
            // 只展示前 4 个装备槽：头盔、胸甲、护腿、靴子。
            // 槽位框已绘制在面板底图上，这里只放透明交互槽位（与设计稿一致，无占位文字）。
            let visible_slots =
                &EquipmentSlot::ALL[..EQUIP_SLOT_COUNT.min(EquipmentSlot::ALL.len())];
            for (index, _equipment_slot) in visible_slots.iter().enumerate() {
                let mut slot_theme = slot_theme(theme, EQUIP_SLOT_WIDTH);
                slot_theme.slot_height = EQUIP_SLOT_HEIGHT;

                spawn_empty_slot(
                    panel,
                    SlotKind::SurvivalEquipment,
                    index,
                    &slot_theme,
                    ui_font,
                );
            }
        });
}

fn spawn_offhand_slot(parent: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    // 副手槽在装备槽列表中的索引
    let offhand_index = EquipmentSlot::ALL
        .iter()
        .position(|s| *s == EquipmentSlot::Offhand)
        .unwrap_or(4);

    let mut slot_theme = slot_theme(theme, OFFHAND_SLOT_SIZE);
    slot_theme.slot_height = OFFHAND_SLOT_SIZE;

    parent
        .spawn((
            SurvivalOffhandSlot,
            Name::new("OffhandSlot"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(OFFHAND_ORIGIN.x),
                top: Val::Px(OFFHAND_ORIGIN.y),
                width: Val::Px(OFFHAND_SLOT_SIZE),
                height: Val::Px(OFFHAND_SLOT_SIZE),
                ..default()
            },
        ))
        .with_children(|slot_container| {
            // 盾牌轮廓已绘制在底图上，这里只放透明交互槽位，避免文字与图标重叠。
            spawn_empty_slot(
                slot_container,
                SlotKind::SurvivalEquipment,
                offhand_index,
                &slot_theme,
                ui_font,
            );
        });
}

/// 生成随身合成区：2×2 输入格 + 1 个输出格。
///
/// 槽位框与箭头已绘制在面板底图上，这里只放置透明交互槽位。
fn spawn_crafting_slots(parent: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    let slot_theme = slot_theme(theme, CRAFT_INPUT_SIZE);
    parent
        .spawn((
            Name::new("CraftingGrid"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CRAFT_GRID_ORIGIN.x),
                top: Val::Px(CRAFT_GRID_ORIGIN.y),
                width: Val::Px(CRAFT_INPUT_SIZE * 2.0 + CRAFT_INPUT_COLUMN_GAP),
                height: Val::Px(CRAFT_INPUT_SIZE * 2.0 + CRAFT_INPUT_ROW_GAP),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(2, CRAFT_INPUT_SIZE),
                grid_template_rows: RepeatedGridTrack::px(2, CRAFT_INPUT_SIZE),
                column_gap: Val::Px(CRAFT_INPUT_COLUMN_GAP),
                row_gap: Val::Px(CRAFT_INPUT_ROW_GAP),
                ..default()
            },
        ))
        .with_children(|grid| {
            for index in 0..4 {
                spawn_empty_slot(
                    grid,
                    SlotKind::Container(ContainerKind::PlayerCrafting),
                    index,
                    &slot_theme,
                    ui_font,
                );
            }
        });

    parent
        .spawn((
            Name::new("CraftingOutput"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CRAFT_OUTPUT_ORIGIN.x),
                top: Val::Px(CRAFT_OUTPUT_ORIGIN.y),
                width: Val::Px(CRAFT_OUTPUT_SIZE),
                height: Val::Px(CRAFT_OUTPUT_SIZE),
                ..default()
            },
        ))
        .with_children(|output| {
            spawn_empty_slot(
                output,
                SlotKind::Container(ContainerKind::PlayerCrafting),
                4,
                &slot_theme,
                ui_font,
            );
        });
}

fn spawn_backpack_grid(parent: &mut ChildSpawnerCommands) {
    let grid_w = BACKPACK_SLOT_SIZE * BACKPACK_COLUMNS as f32
        + BACKPACK_COLUMN_GAP * (BACKPACK_COLUMNS - 1) as f32;
    let grid_h =
        BACKPACK_SLOT_SIZE * BACKPACK_ROWS as f32 + BACKPACK_ROW_GAP * (BACKPACK_ROWS - 1) as f32;

    parent.spawn((
        SurvivalItemGrid,
        Name::new("SurvivalGrid"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(BACKPACK_GRID_ORIGIN.x),
            top: Val::Px(BACKPACK_GRID_ORIGIN.y),
            width: Val::Px(grid_w),
            height: Val::Px(grid_h),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(
                BACKPACK_COLUMNS as u16,
                BACKPACK_SLOT_SIZE,
            ),
            grid_template_rows: RepeatedGridTrack::px(BACKPACK_ROWS as u16, BACKPACK_SLOT_SIZE),
            column_gap: Val::Px(BACKPACK_COLUMN_GAP),
            row_gap: Val::Px(BACKPACK_ROW_GAP),
            ..default()
        },
    ));
}

fn spawn_hotbar_panel(parent: &mut ChildSpawnerCommands) {
    let bar_w = HOTBAR_SLOT_WIDTH * 9.0 + HOTBAR_SLOT_GAP * 8.0;
    parent.spawn((
        SurvivalHotbarPanel,
        Name::new("SurvivalHotbar"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(HOTBAR_ORIGIN.x),
            top: Val::Px(HOTBAR_ORIGIN.y),
            width: Val::Px(bar_w),
            height: Val::Px(HOTBAR_SLOT_HEIGHT),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(HOTBAR_SLOT_GAP),
            ..default()
        },
    ));
}

/// 根据槽位尺寸返回生存背包使用的局部视觉主题。
///
/// 槽位框已绘制在面板底图中，Bevy 槽位只承载图标与交互，
/// 背景与边框全部透明，避免彩色矩形盖住像素风底图。
pub(super) fn slot_theme(theme: &UiTheme, size: f32) -> UiTheme {
    let mut result = theme.clone();
    result.slot_size = size;
    result.slot_border = 0.0;
    result.bg_slot = Color::NONE;
    result.border_default = Color::NONE;
    result.border_selected = Color::NONE;
    result
}
