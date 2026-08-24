//! Toast 的节点生成、消息消费与生命周期推进。

use bevy::prelude::*;

use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::game::notification::{NotificationLevel, PlayerNotification};

use super::components::{ToastItem, ToastRoot};
use super::queue::ToastQueue;

/// Toast 完整显示时长（秒）。
const TOAST_VISIBLE_SECONDS: f32 = 3.0;
/// Toast 淡出过渡时长（秒）。
const TOAST_FADE_SECONDS: f32 = 0.5;
/// Toast 堆叠区域宽度（像素）；文本在宽度内自动换行。
const TOAST_STACK_WIDTH: f32 = 360.0;
/// Toast 文本字号。
const TOAST_FONT_SIZE: f32 = 18.0;

/// 在 Startup 生成常驻的 Toast 堆叠容器，锚定屏幕右上角。
pub fn spawn_toast_root_system(mut commands: Commands) {
    commands.spawn((
        ToastRoot,
        Name::new("ToastRoot"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            top: Val::Px(16.0),
            width: Val::Px(TOAST_STACK_WIDTH),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(8.0),
            ..default()
        },
        GlobalZIndex(5500),
    ));
}

/// 消费通知消息并入队，再按剩余容量生成 Toast 节点；最新通知显示在最上方。
pub fn push_toast_system(
    mut notifications: MessageReader<PlayerNotification>,
    mut queue: ResMut<ToastQueue>,
    root_query: Query<Entity, With<ToastRoot>>,
    item_query: Query<Entity, With<ToastItem>>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    mut commands: Commands,
) {
    for notification in notifications.read() {
        queue.push(notification.clone());
    }
    let Ok(root) = root_query.single() else {
        return;
    };
    let active_count = item_query.iter().count();
    for notification in queue.admit(active_count) {
        let entity = commands
            .spawn((
                ToastItem::new(
                    TOAST_VISIBLE_SECONDS,
                    TOAST_FADE_SECONDS,
                    theme.bg_panel.alpha(),
                ),
                Name::new("ToastItem"),
                Node {
                    padding: UiRect {
                        left: Val::Px(12.0),
                        right: Val::Px(12.0),
                        top: Val::Px(8.0),
                        bottom: Val::Px(8.0),
                    },
                    border: UiRect::left(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(theme.bg_panel),
                BorderColor::all(level_accent(notification.level, &theme)),
                Text::new(notification.text.clone()),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(TOAST_FONT_SIZE),
                    ..default()
                },
                TextColor(theme.text_primary),
            ))
            .id();
        // 插入到堆叠首位，使最新通知位于最上方。
        commands.entity(root).insert_children(0, &[entity]);
    }
}

/// 推进每条 Toast 的生命周期：显示计时到期后整块淡出，淡出结束回收实体。
pub fn update_toast_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut ToastItem,
        &mut TextColor,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for (entity, mut item, mut text, mut background, border) in &mut query {
        if !item.fading {
            item.timer.tick(time.delta());
            if item.timer.is_finished() {
                item.fading = true;
            }
        } else {
            item.fade_timer.tick(time.delta());
            let alpha = 1.0 - item.fade_timer.fraction();
            text.0.set_alpha(alpha);
            background.0.set_alpha(item.base_bg_alpha * alpha);
            // into_inner 解出原生引用，四个边框字段可分别可变借用。
            let border = border.into_inner();
            for color in [
                &mut border.top,
                &mut border.right,
                &mut border.bottom,
                &mut border.left,
            ] {
                color.set_alpha(alpha);
            }
            if item.fade_timer.is_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// 按通知级别返回左侧强调条颜色。
fn level_accent(level: NotificationLevel, theme: &UiTheme) -> Color {
    match level {
        NotificationLevel::Info => theme.accent,
        NotificationLevel::Success => Color::srgba(0.42, 0.78, 0.45, 1.0),
        NotificationLevel::Warning => Color::srgba(0.95, 0.70, 0.25, 1.0),
    }
}
