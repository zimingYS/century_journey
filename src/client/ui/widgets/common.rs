//! 提供按钮、文本和面板等可复用界面组件包。

use bevy::ecs::bundle::Bundle;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;

/// 通用界面控件的视觉和交互类别。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiControlKind {
    Button,
    IconButton,
    Tab,
    /// 固定宽度的文字动作按钮，与步进图标按钮同宽以便设置行右列对齐。
    Toggle,
}

/// 通用控件当前的选中和禁用状态。
#[derive(Component, Debug, Clone, Copy)]
pub struct UiControl {
    pub kind: UiControlKind,
    pub selected: bool,
    pub disabled: bool,
}

/// 可由鼠标滚轮驱动的界面滚动区域标记。
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiScrollArea;

/// 使用模态框架样式的界面面板标记。
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiModalPanel;

/// 创建带文本标签和通用交互状态的主题按钮。
pub fn spawn_text_button<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    label: &str,
    kind: UiControlKind,
    theme: &UiTheme,
    ui_font: &UiFont,
) -> Entity {
    parent
        .spawn((
            marker,
            UiControl {
                kind,
                selected: false,
                disabled: false,
            },
            Button,
            Pickable::default(),
            control_node(kind, theme),
            BackgroundColor(theme.tab_active_bg),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(match kind {
                        UiControlKind::IconButton => theme.title_font_size,
                        UiControlKind::Button | UiControlKind::Tab | UiControlKind::Toggle => {
                            theme.body_font_size
                        }
                    }),
                    ..default()
                },
                TextColor(theme.text_primary),
                Pickable::IGNORE,
            ));
        })
        .id()
}

/// 创建固定尺寸图标按钮；图标由字体字符提供。
pub fn spawn_icon_button<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    icon: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
) -> Entity {
    spawn_text_button(
        parent,
        marker,
        icon,
        UiControlKind::IconButton,
        theme,
        ui_font,
    )
}

/// 创建可声明初始选中状态的标签页按钮。
pub fn spawn_tab<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    label: &str,
    selected: bool,
    theme: &UiTheme,
    ui_font: &UiFont,
) -> Entity {
    let entity = spawn_text_button(parent, marker, label, UiControlKind::Tab, theme, ui_font);
    parent.commands().entity(entity).insert(UiControl {
        kind: UiControlKind::Tab,
        selected,
        disabled: false,
    });
    entity
}

/// 创建仅纵向滚动的主题滚动区域。
pub fn spawn_scroll_area<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    node: Node,
) -> Entity {
    parent
        .spawn((
            marker,
            UiScrollArea,
            Interaction::None,
            Pickable::default(),
            ScrollPosition::default(),
            Node {
                overflow: Overflow::scroll_y(),
                min_height: Val::Px(0.0),
                ..node
            },
        ))
        .id()
}

/// 将滚轮输入应用到当前悬停的滚动区域。
pub fn scroll_area_wheel_system(
    mut wheel_events: MessageReader<MouseWheel>,
    mut query: Query<(&Interaction, &mut ScrollPosition), With<UiScrollArea>>,
) {
    let delta = wheel_events
        .read()
        .map(|event| match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 32.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        })
        .sum::<f32>();
    if delta == 0.0 {
        return;
    }
    if let Some((_, mut position)) = query
        .iter_mut()
        .find(|(interaction, _)| **interaction == Interaction::Hovered)
    {
        position.0.y = (position.0.y - delta).max(0.0);
    }
}

/// 创建带统一内边距和九宫格边框的模态面板。
pub fn spawn_modal_panel<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    node: Node,
    theme: &UiTheme,
) -> Entity {
    parent
        .spawn((
            marker,
            UiModalPanel,
            UiFrameKind::Modal,
            Node {
                padding: UiRect::all(Val::Px(theme.spacing_lg)),
                border: UiRect::all(Val::Px(2.0)),
                ..node
            },
            BackgroundColor(theme.bg_panel),
            BorderColor::all(theme.border_default),
        ))
        .id()
}

/// 根据悬停、按下、选中和禁用状态同步控件主题颜色。
/// 查询元组同时更新控件背景和边框，过滤器仅接受发生视觉状态变化的按钮。
#[allow(clippy::type_complexity)]
pub fn themed_control_interaction_system(
    theme: Res<UiTheme>,
    mut query: Query<
        (
            &Interaction,
            &UiControl,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Or<(Changed<Interaction>, Changed<UiControl>)>, With<Button>),
    >,
) {
    for (interaction, control, mut background, mut border) in &mut query {
        let (bg, outline) = if control.disabled {
            (theme.bg_content, theme.border_default)
        } else {
            match interaction {
                Interaction::Pressed => (theme.control_pressed, theme.border_selected),
                Interaction::Hovered => (theme.control_hover, theme.border_hover),
                Interaction::None if control.selected => (theme.accent, theme.border_selected),
                Interaction::None => (theme.tab_active_bg, theme.border_default),
            }
        };
        *background = BackgroundColor(bg);
        *border = BorderColor::all(outline);
    }
}

fn control_node(kind: UiControlKind, theme: &UiTheme) -> Node {
    // Tab 宽度用 Auto：横向页签栏中按内容自适应避免溢出；
    // 纵向列表里依赖交叉轴默认 stretch 依旧占满整行。
    // IconButton 与 Toggle 固定同宽：设置页步进行与开关行的数值列由此纵向对齐。
    let (width, height) = match kind {
        UiControlKind::IconButton | UiControlKind::Toggle => (Val::Px(56.0), Val::Px(40.0)),
        UiControlKind::Button => (Val::Auto, Val::Px(40.0)),
        UiControlKind::Tab => (Val::Auto, Val::Px(theme.tab_height)),
    };
    Node {
        width,
        height,
        min_width: Val::Px(40.0),
        flex_shrink: 0.0,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(theme.spacing_md)),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}
