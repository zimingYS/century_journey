use crate::client::ui::hud::bottom::bars::{
    HUD_STATUS_ICON_GAP, HudStatusIconAssets, LeftBarsHud, shown_status_units, status_icon_count,
    status_icon_node, status_icon_segment,
};
use crate::game::player::components::Player;
use crate::game::player::components::stats::Health;
use bevy::prelude::*;

/// 生命值条根节点。
#[derive(Component)]
pub struct HealthBar;

/// 生命值条当前已经绘制的状态。
#[derive(Component, Default, PartialEq, Eq)]
pub struct HealthBarVisual {
    /// 当前显示的半格生命数量。
    shown_units: u32,
    /// 当前显示的生命图标格数量。
    icon_count: usize,
}

/// 生成生命值 HUD。
pub fn spawn_health_bar(mut commands: Commands, bars_hud: Query<Entity, With<LeftBarsHud>>) {
    let Ok(bars_hud_entity) = bars_hud.single() else {
        log::error!("LEFT BARS HUD NOT FOUND - cannot spawn health bar");
        return;
    };

    commands.entity(bars_hud_entity).with_children(|parent| {
        parent.spawn((
            HealthBar,
            HealthBarVisual::default(),
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(HUD_STATUS_ICON_GAP),
                ..default()
            },
        ));
    });
}

/// 同步生命值图标显示。
pub fn health_bar_sync_system(
    health_query: Query<&Health, With<Player>>,
    mut bar_query: Query<(Entity, &mut HealthBarVisual), With<HealthBar>>,
    children_query: Query<&Children>,
    icons: Res<HudStatusIconAssets>,
    mut commands: Commands,
) {
    let Ok(health) = health_query.single() else {
        return;
    };
    let Ok((bar_entity, mut visual)) = bar_query.single_mut() else {
        return;
    };

    let shown_units = shown_status_units(health.current, health.max);
    let icon_count = status_icon_count(health.max);
    if visual.shown_units == shown_units && visual.icon_count == icon_count {
        return;
    }

    if let Ok(children) = children_query.get(bar_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    commands.entity(bar_entity).with_children(|bar| {
        for index in 0..icon_count {
            let segment = status_icon_segment(shown_units, index);
            bar.spawn(status_icon_node(icons.heart_icon(segment)));
        }
    });

    visual.shown_units = shown_units;
    visual.icon_count = icon_count;
}
