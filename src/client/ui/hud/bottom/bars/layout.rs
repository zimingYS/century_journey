//! 底部状态条的几何布局与图标分格计算。

use bevy::prelude::*;

use crate::client::ui::theme::ui_theme::UiTheme;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;

/// HUD 状态图标的显示尺寸。
pub const HUD_STATUS_ICON_SIZE: f32 = 24.0;
/// HUD 状态图标之间的间距。
pub const HUD_STATUS_ICON_GAP: f32 = 2.0;
/// HUD 快捷栏外框内边距，需与 hotbar.rs 保持一致。
const HUD_HOTBAR_PADDING: f32 = 4.0;
/// HUD 快捷栏外框边框宽度，需与 hotbar.rs 保持一致。
const HUD_HOTBAR_BORDER: f32 = 2.0;

/// 计算 HUD 快捷栏外框的视觉宽度。
///
/// 状态条使用这个宽度后，生命条左边会对齐快捷栏左边，饥饿条右边会对齐快捷栏右边。
pub fn hud_hotbar_outer_width(theme: &UiTheme) -> f32 {
    let slot_count = HOTBAR_SIZE as f32;
    let gap_count = HOTBAR_SIZE.saturating_sub(1) as f32;
    slot_count * theme.slot_size
        + gap_count * theme.slot_gap
        + HUD_HOTBAR_PADDING * 2.0
        + HUD_HOTBAR_BORDER * 2.0
}

/// 计算最大值需要显示多少个图标格。
pub fn status_icon_count(max_value: f32) -> usize {
    (max_value.max(0.0) / 2.0).ceil() as usize
}

/// 把当前值转换为以半格为单位的显示数量。
pub fn shown_status_units(current_value: f32, max_value: f32) -> u32 {
    current_value.clamp(0.0, max_value.max(0.0)).ceil() as u32
}

/// 根据半格数量和图标序号计算该图标应该显示满格、半格还是空格。
pub fn status_icon_segment(shown_units: u32, index: usize) -> super::resources::StatusIconSegment {
    let slot_start = index as u32 * 2;
    let remaining = shown_units.saturating_sub(slot_start);
    if remaining >= 2 {
        super::resources::StatusIconSegment::Full
    } else if remaining == 1 {
        super::resources::StatusIconSegment::Half
    } else {
        super::resources::StatusIconSegment::Empty
    }
}

/// 创建一个 HUD 状态图标节点。
pub fn status_icon_node(image: Handle<Image>) -> impl Bundle {
    (
        ImageNode {
            image,
            texture_atlas: None,
            ..default()
        },
        Node {
            width: Val::Px(HUD_STATUS_ICON_SIZE),
            height: Val::Px(HUD_STATUS_ICON_SIZE),
            ..default()
        },
    )
}
