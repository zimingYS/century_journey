use crate::client::ui::hud::bottom::BottomHud;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::engine::asset::identifier::asset_id;
use crate::engine::asset::manager::AssetManager;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use bevy::prelude::*;

pub mod left_bars;
pub mod right_bars;

/// HUD 状态图标的显示尺寸。
pub const HUD_STATUS_ICON_SIZE: f32 = 24.0;
/// HUD 状态图标之间的间距。
pub const HUD_STATUS_ICON_GAP: f32 = 2.0;
/// HUD 快捷栏外框内边距，需与 hotbar.rs 保持一致。
const HUD_HOTBAR_PADDING: f32 = 4.0;
/// HUD 快捷栏外框边框宽度，需与 hotbar.rs 保持一致。
const HUD_HOTBAR_BORDER: f32 = 2.0;

/// 底部状态条总容器。
#[derive(Component)]
pub struct BarsHud;

/// 左侧状态条容器，目前承载护甲和生命值。
#[derive(Component)]
pub struct LeftBarsHud;

/// 右侧状态条容器，目前承载饥饿值。
#[derive(Component)]
pub struct RightBarsHud;

/// HUD 状态图标缓存。
///
/// 这里统一保存生命值和饥饿值的 full / half / empty 图标句柄，避免同步系统反复拼路径或直接加载资源。
#[derive(Resource, Default, Clone)]
pub struct HudStatusIconAssets {
    /// 满生命图标。
    heart_full: Handle<Image>,
    /// 半生命图标。
    heart_half: Handle<Image>,
    /// 空生命图标。
    heart_empty: Handle<Image>,
    /// 满饥饿值图标。
    hunger_full: Handle<Image>,
    /// 半饥饿值图标。
    hunger_half: Handle<Image>,
    /// 空饥饿值图标。
    hunger_empty: Handle<Image>,
}

impl HudStatusIconAssets {
    /// 根据生命格状态取得对应图片。
    pub fn heart_icon(&self, segment: StatusIconSegment) -> Handle<Image> {
        match segment {
            StatusIconSegment::Full => self.heart_full.clone(),
            StatusIconSegment::Half => self.heart_half.clone(),
            StatusIconSegment::Empty => self.heart_empty.clone(),
        }
    }

    /// 根据饥饿格状态取得对应图片。
    pub fn hunger_icon(&self, segment: StatusIconSegment) -> Handle<Image> {
        match segment {
            StatusIconSegment::Full => self.hunger_full.clone(),
            StatusIconSegment::Half => self.hunger_half.clone(),
            StatusIconSegment::Empty => self.hunger_empty.clone(),
        }
    }
}

/// 单个状态格的显示状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIconSegment {
    /// 该格为空。
    Empty,
    /// 该格显示半格。
    Half,
    /// 该格显示满格。
    Full,
}

/// 加载 HUD 状态图标资源。
///
/// 通过 AssetManager::texture 加载可以复用项目的像素纹理加载规则，保持最近邻采样。
pub fn load_hud_status_icon_assets_system(
    mut icons: ResMut<HudStatusIconAssets>,
    mut asset_manager: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
) {
    icons.heart_full = asset_manager
        .texture(&asset_id("textures/ui/hud/heart_full"), &asset_server)
        .handle;
    icons.heart_half = asset_manager
        .texture(&asset_id("textures/ui/hud/heart_half"), &asset_server)
        .handle;
    icons.heart_empty = asset_manager
        .texture(&asset_id("textures/ui/hud/heart_empty"), &asset_server)
        .handle;
    icons.hunger_full = asset_manager
        .texture(&asset_id("textures/ui/hud/hunger_full"), &asset_server)
        .handle;
    icons.hunger_half = asset_manager
        .texture(&asset_id("textures/ui/hud/hunger_half"), &asset_server)
        .handle;
    icons.hunger_empty = asset_manager
        .texture(&asset_id("textures/ui/hud/hunger_empty"), &asset_server)
        .handle;
}

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
pub fn status_icon_segment(shown_units: u32, index: usize) -> StatusIconSegment {
    let slot_start = index as u32 * 2;
    let remaining = shown_units.saturating_sub(slot_start);
    if remaining >= 2 {
        StatusIconSegment::Full
    } else if remaining == 1 {
        StatusIconSegment::Half
    } else {
        StatusIconSegment::Empty
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

/// 生成底部状态条容器。
pub fn spawn_bars_hud_system(
    mut commands: Commands,
    bottom_hud: Query<Entity, With<BottomHud>>,
    theme: Res<UiTheme>,
) {
    let Ok(bottom_entity) = bottom_hud.single() else {
        log::error!("BOTTOM HUD NOT FOUND - cannot spawn BarsHud");
        return;
    };

    commands.entity(bottom_entity).with_children(|parent| {
        parent
            .spawn((
                BarsHud,
                Name::new("BarsHud"),
                Node {
                    width: Val::Px(hud_hotbar_outer_width(&theme)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|bars| {
                bars.spawn((
                    LeftBarsHud,
                    Name::new("LeftBarsHud"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ));
                bars.spawn((
                    RightBarsHud,
                    Name::new("RightBarsHud"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ));
            });
    });
}
