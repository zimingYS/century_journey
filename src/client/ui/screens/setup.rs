//! 构建常驻界面根节点、通用覆盖层和屏幕级容器。

use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::SelectAllOnFocus;

use crate::client::ui::components::{
    CreativeCategoryPanel, CreativeCloseButton, CreativeHotbarPanel, CreativeInventoryOverlay,
    CreativeInventoryRoot, CreativeItemGrid, CreativeRecentGrid, CreativeRecentPanel,
    CreativeSearchBox, CreativeSearchPlaceholder, CreativeTabPagerLeft, CreativeTabPagerRight,
    CreativeTabPagerText, CreativeTitleIcon,
};
use crate::client::ui::localization::LocalizedText;
use crate::client::ui::navigation::{UiScreenAudience, UiScreenRoot};
use crate::client::ui::resources::creative_assets::{
    CreativeUiAssets, SEARCH_BOX_SLICE, sliced_image_node,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::CreativeSearchInput;
use crate::engine::localization::Localization;

/// 创造物品栏位于 HUD 之上，避免被底部快捷栏盖住。
const CREATIVE_OVERLAY_Z: i32 = 1000;
/// 面板显示尺寸：设计稿 1529x1017 等比缩放，整图拉伸因此内部装饰不变形。
const CREATIVE_PANEL_WIDTH: f32 = 1200.0;
const CREATIVE_PANEL_HEIGHT: f32 = 800.0;
/// 顶栏高度：设计稿标题区含底部分隔线约 110px。
const CREATIVE_HEADER_HEIGHT: f32 = 83.0;
/// 左侧分类栏宽度：设计稿分类区约 285px。
const CREATIVE_SIDEBAR_WIDTH: f32 = 224.0;
/// 右侧最近使用栏宽度：设计稿辅助区约 229px。
const CREATIVE_RECENT_WIDTH: f32 = 196.0;
/// 中部网格列数
const CREATIVE_GRID_COLUMNS: u16 = 9;
/// 网格与最近使用面板的槽位间距（设计稿约 3px）。
pub(super) const CREATIVE_SLOT_GAP: f32 = 3.0;
/// 快捷栏槽位间距（设计稿约 2px，槽位间几乎相接）。
pub(super) const CREATIVE_HOTBAR_SLOT_GAP: f32 = 2.0;
/// 分类标签列表的页容量；设计稿单页恰好展示十个分类。
pub(super) const CREATIVE_TABS_PER_PAGE: usize = 10;
/// 快捷栏区域高度：设计稿分隔线以下到底边框约 210px。
const CREATIVE_HOTBAR_HEIGHT: f32 = 158.0;
/// 搜索框显示尺寸（素材 461x68 等比）。
const CREATIVE_SEARCH_WIDTH: f32 = 360.0;
const CREATIVE_SEARCH_HEIGHT: f32 = 80.0;
/// 搜索图标显示尺寸（素材 39x40 等比）。
const CREATIVE_SEARCH_ICON_WIDTH: f32 = 29.0;
/// 关闭按钮显示尺寸（素材 56x57 等比）。
const CREATIVE_CLOSE_WIDTH: f32 = 64.0;
const CREATIVE_CLOSE_HEIGHT: f32 = 64.0;
/// 标题图标尺寸；与标题文字行高协调。
const CREATIVE_TITLE_ICON_SIZE: f32 = 64.0;
/// 最近使用面板标题高度。
const CREATIVE_RECENT_TITLE_HEIGHT: f32 = 64.0;
/// 最近使用面板底部箱子按钮尺寸（设计稿约 105x103px 等比）。
const CREATIVE_CHEST_BUTTON_SIZE: f32 = 78.0;

/// 构造创造模式物品栏 UI。
///
/// 面板背景为设计稿整图拉伸，内容区按设计稿比例排布以贴合纹理中的
/// 分隔线与区域描边；纹理资源由 Startup 链前部的加载系统注入。
pub fn spawn_creative_inventory_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    localization: Res<Localization>,
    assets: Res<CreativeUiAssets>,
) {
    commands
        .spawn((
            CreativeInventoryOverlay,
            UiScreenRoot::inventory(UiScreenAudience::Creative),
            Name::new("CreativeOverlay"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(18.0)),
                ..default()
            },
            // 明确提高创造物品栏层级，保证它渲染在 HUD 快捷栏之上。
            ZIndex(CREATIVE_OVERLAY_Z),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.46)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    CreativeInventoryRoot,
                    Name::new("CreativeRoot"),
                    Node {
                        width: Val::Px(CREATIVE_PANEL_WIDTH),
                        height: Val::Px(CREATIVE_PANEL_HEIGHT),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    // 面板整图拉伸：比例与设计稿一致，木框和区域描边保持原样。
                    ImageNode {
                        image: assets.panel.clone(),
                        ..default()
                    },
                ))
                .with_children(|root| {
                    build_header(root, &ui_font, &theme, &localization, &assets);
                    build_inventory_frame(root, &ui_font, &theme, &localization, &assets);
                });
        });
}

/// 构造标题栏、搜索框与关闭按钮。
fn build_header(
    root: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    theme: &UiTheme,
    localization: &Localization,
    assets: &CreativeUiAssets,
) {
    root.spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Px(CREATIVE_HEADER_HEIGHT),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        padding: UiRect::new(Val::Px(32.0), Val::Px(32.0), Val::Px(16.0), Val::Px(0.0)),
        column_gap: Val::Px(32.0),
        ..default()
    },))
        .with_children(|header| {
            header
                .spawn((Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },))
                .with_children(|title| {
                    // 标题图标节点先占位，草方块 3D 图标由皮肤系统待渲染资源就绪后填充。
                    title.spawn((
                        CreativeTitleIcon,
                        Node {
                            width: Val::Px(CREATIVE_TITLE_ICON_SIZE),
                            height: Val::Px(CREATIVE_TITLE_ICON_SIZE),
                            ..default()
                        },
                    ));
                    title.spawn((
                        LocalizedText::new("creative.title"),
                        Text::new(localization.get("creative.title")),
                        TextFont {
                            font: FontSource::from(ui_font.default.clone()),
                            font_size: FontSize::Px(32.0),
                            ..default()
                        },
                        TextColor(theme.text_primary),
                    ));
                });

            header
                .spawn((Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                },))
                .with_children(|right| {
                    build_search_box(right, ui_font, theme, localization, assets);
                    build_close_button(right, assets);
                });
        });
}

/// 构造搜索框。占位文字是单独节点，不会污染真实搜索文本。
fn build_search_box(
    parent: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    theme: &UiTheme,
    localization: &Localization,
    assets: &CreativeUiAssets,
) {
    parent
        .spawn((
            CreativeSearchBox,
            Name::new("CreativeSearchBox"),
            Node {
                width: Val::Px(CREATIVE_SEARCH_WIDTH),
                height: Val::Px(CREATIVE_SEARCH_HEIGHT),
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(24.0), Val::Px(0.0), Val::Px(24.0), Val::Px(16.0)),
                column_gap: Val::Px(8.0),
                overflow: Overflow::clip_x(),
                ..default()
            },
            sliced_image_node(assets.search_box.clone(), SEARCH_BOX_SLICE),
        ))
        .with_children(|search| {
            search.spawn((
                ImageNode {
                    image: assets.search_icon.clone(),
                    ..default()
                },
                Node {
                    padding: UiRect::new(Val::Px(8.0), Val::Px(1.0), Val::Px(1.0), Val::Px(1.0)),
                    width: Val::Px(CREATIVE_SEARCH_ICON_WIDTH),
                    height: Val::Percent(100.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            search.spawn((
                CreativeSearchPlaceholder,
                LocalizedText::new("creative.search-placeholder"),
                Text::new(localization.get("creative.search-placeholder")),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.search_font_size + 3.0),
                    ..default()
                },
                TextColor(Color::srgba(0.62, 0.62, 0.64, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(64.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            search.spawn((
                CreativeSearchInput,
                Name::new("CreativeSearchInput"),
                EditableText {
                    visible_width: Some(18.0),
                    max_characters: Some(64),
                    allow_newlines: false,
                    ..default()
                },
                TextCursorStyle {
                    color: theme.text_primary,
                    selection_color: theme.border_selected,
                    unfocused_selection_color: theme.border_hover,
                    selected_text_color: Some(Color::BLACK),
                },
                SelectAllOnFocus,
                TextLayout::no_wrap(),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.search_font_size + 3.0),
                    ..default()
                },
                TextColor(theme.text_primary),
                Node {
                    flex_grow: 1.0,
                    // height: Val::Percent(128.0),
                    padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(1.0), Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
        });
}

/// 构造右上角关闭按钮。
fn build_close_button(parent: &mut ChildSpawnerCommands, assets: &CreativeUiAssets) {
    parent.spawn((
        CreativeCloseButton,
        Button,
        Pickable::default(),
        Node {
            width: Val::Px(CREATIVE_CLOSE_WIDTH),
            height: Val::Px(CREATIVE_CLOSE_HEIGHT),
            padding: UiRect::new(Val::Px(1.0), Val::Px(16.0), Val::Px(8.0), Val::Px(1.0)),
            ..default()
        },
        ImageNode {
            image: assets.close.clone(),
            ..default()
        },
    ));
}

/// 构造三栏主体：左分类、中间物品、右最近使用。
fn build_inventory_frame(
    root: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    theme: &UiTheme,
    localization: &Localization,
    assets: &CreativeUiAssets,
) {
    root.spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Px(0.0),
        min_height: Val::Px(0.0),
        flex_grow: 1.0,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Stretch,
        overflow: Overflow::clip_y(),
        ..default()
    },))
        .with_children(|frame| {
            build_category_sidebar(frame, ui_font, theme, assets);
            build_center_panel(frame);
            build_recent_panel(frame, ui_font, theme, localization);
        });
}

/// 构造左侧分类栏和页码区。
fn build_category_sidebar(
    parent: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    theme: &UiTheme,
    assets: &CreativeUiAssets,
) {
    parent
        .spawn((Node {
            width: Val::Px(CREATIVE_SIDEBAR_WIDTH),
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::new(Val::Px(30.0), Val::Px(0.0), Val::Px(30.0), Val::Px(0.0)),
            overflow: Overflow::clip_y(),
            ..default()
        },))
        .with_children(|sidebar| {
            sidebar.spawn((
                CreativeCategoryPanel,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
            ));
            build_tab_pager(sidebar, ui_font, theme, assets);
        });
}

/// 左/右翻页按钮显示尺寸（设计稿素材 49x60，按面板 1150/1529、766/1017 等比换算）。
const CREATIVE_PAGER_BUTTON_WIDTH: f32 = 36.9;
const CREATIVE_PAGER_BUTTON_HEIGHT: f32 = 45.2;
/// 翻页行距侧栏底边的距离：设计稿按钮底边 y=959 映射到面板 y=722.3，侧栏底为 756。
const CREATIVE_PAGER_BOTTOM_MARGIN: f32 = 24.0;
/// 左按钮距侧栏左缘：设计稿 x=59 映射到面板 x=44.4，减去侧栏左缘 12。
const CREATIVE_PAGER_LEFT_PADDING: f32 = 24.0;
/// 右按钮距侧栏右缘：设计稿右缘 x=272 映射到面板 x=204.6，侧栏右缘为 226。
const CREATIVE_PAGER_RIGHT_PADDING: f32 = 21.4;

/// 构造分类列表底部的翻页行。
///
/// 按钮位置对齐面板纹理中的装饰线区域：设计稿中按钮垂直居中于
/// y∈[879,979] 的装饰带内，页码文本与按钮垂直居中。
fn build_tab_pager(
    parent: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    theme: &UiTheme,
    assets: &CreativeUiAssets,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(CREATIVE_PAGER_BUTTON_HEIGHT),
            margin: UiRect::bottom(Val::Px(CREATIVE_PAGER_BOTTOM_MARGIN)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::new(
                Val::Px(CREATIVE_PAGER_LEFT_PADDING),
                Val::Px(CREATIVE_PAGER_RIGHT_PADDING),
                Val::Px(0.0),
                Val::Px(45.0),
            ),
            ..default()
        },))
        .with_children(|pager| {
            build_pager_button(pager, CreativeTabPagerLeft, assets.pager_left.clone());
            pager.spawn((
                CreativeTabPagerText,
                Text::new("1 / 1"),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.body_font_size + 2.0),
                    ..default()
                },
                TextColor(theme.text_primary),
                Pickable::IGNORE,
            ));
            build_pager_button(pager, CreativeTabPagerRight, assets.pager_right.clone());
        });
}

/// 构造单个翻页箭头按钮，使用纹理图标替代文字箭头。
fn build_pager_button<M: bevy::ecs::bundle::Bundle>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    image: Handle<Image>,
) {
    parent.spawn((
        marker,
        Button,
        Pickable::default(),
        Node {
            width: Val::Px(CREATIVE_PAGER_BUTTON_WIDTH),
            height: Val::Px(CREATIVE_PAGER_BUTTON_HEIGHT),
            ..default()
        },
        ImageNode { image, ..default() },
    ));
}

/// 构造中间物品网格和面板内快捷栏。
fn build_center_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((Node {
            min_height: Val::Px(0.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip_y(),
            ..default()
        },))
        .with_children(|center| {
            center.spawn((
                CreativeItemGrid,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    flex_grow: 1.0,
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(CREATIVE_GRID_COLUMNS, 1.0),
                    column_gap: Val::Px(CREATIVE_SLOT_GAP),
                    row_gap: Val::Px(CREATIVE_SLOT_GAP),
                    padding: UiRect::new(
                        Val::Px(32.0),
                        Val::Px(32.0),
                        Val::Px(32.0),
                        Val::Px(16.0),
                    ),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ));
            center.spawn((
                CreativeHotbarPanel,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(CREATIVE_HOTBAR_HEIGHT),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(CREATIVE_HOTBAR_SLOT_GAP),
                    padding: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(0.0), Val::Px(30.0)),
                    ..default()
                },
            ));
        });
}

/// 构造右侧最近使用栏：标题、换行槽位容器与底部箱子按钮。
fn build_recent_panel(
    parent: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    theme: &UiTheme,
    localization: &Localization,
) {
    parent
        .spawn((
            CreativeRecentPanel,
            Node {
                width: Val::Px(CREATIVE_RECENT_WIDTH),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                LocalizedText::new("creative.recent"),
                Text::new(localization.get("creative.recent")),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.body_font_size + 2.0),
                    ..default()
                },
                TextColor(theme.text_primary),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(CREATIVE_RECENT_TITLE_HEIGHT),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::new(Val::Px(54.0), Val::Px(0.0), Val::Px(32.0), Val::Px(0.0)),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            panel.spawn((
                CreativeRecentGrid,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::FlexStart,
                    column_gap: Val::Px(5.0),
                    row_gap: Val::Px(3.0),
                    overflow: Overflow::clip_y(),
                    padding: UiRect::new(Val::Px(0.0), Val::Px(24.0), Val::Px(0.0), Val::Px(0.0)),
                    ..default()
                },
            ));
            build_chest_button(panel, ui_font, localization);
        });
}

/// 底部箱子按钮是视觉占位，后续可接入保存/加载创造热键栏。
fn build_chest_button(
    parent: &mut ChildSpawnerCommands,
    ui_font: &UiFont,
    localization: &Localization,
) {
    parent
        .spawn((Node {
            width: Val::Px(CREATIVE_CHEST_BUTTON_SIZE),
            height: Val::Px(CREATIVE_CHEST_BUTTON_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(64.0), Val::Px(0.0)),
            ..default()
        },))
        .with_children(|slot| {
            slot.spawn((
                LocalizedText::new("creative.chest"),
                Text::new(localization.get("creative.chest")),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(34.0),
                    ..default()
                },
                TextColor(Color::srgba(0.78, 0.66, 0.42, 1.0)),
                Pickable::IGNORE,
            ));
        });
}
