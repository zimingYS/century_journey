use crate::client::ui::hud::bottom::bars::{
    HUD_STATUS_ICON_GAP, HudStatusIconAssets, RightBarsHud, shown_status_units, status_icon_count,
    status_icon_node, status_icon_segment,
};
use crate::game::player::components::Player;
use crate::game::player::components::stats::Hunger;
use bevy::prelude::*;

/// 饥饿值条根节点。
#[derive(Component)]
pub struct HungerBar;

/// 饥饿值条当前已经绘制的状态。
#[derive(Component, Default, PartialEq, Eq)]
pub struct HungerBarVisual {
    /// 当前显示的半格饥饿数量。
    shown_units: u32,
    /// 当前显示的饥饿图标格数量。
    icon_count: usize,
}

/// 生成饥饿值 HUD。
pub fn spawn_hunger_bar(mut commands: Commands, bars_hud: Query<Entity, With<RightBarsHud>>) {
    let Ok(bars_hud_entity) = bars_hud.single() else {
        log::error!("BARS HUD NOT FOUND - cannot spawn hunger bar");
        return;
    };

    commands.entity(bars_hud_entity).with_children(|parent| {
        parent.spawn((
            HungerBar,
            HungerBarVisual::default(),
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(HUD_STATUS_ICON_GAP),
                ..default()
            },
        ));
    });
}

/// 根据饥饿值同步 HUD 图标显示。
pub fn hunger_bar_sync_system(
    hunger_query: Query<&Hunger, With<Player>>,
    mut bar_query: Query<(Entity, &mut HungerBarVisual), With<HungerBar>>,
    children_query: Query<&Children>,
    icons: Res<HudStatusIconAssets>,
    mut commands: Commands,
) {
    let Ok(hunger) = hunger_query.single() else {
        return;
    };
    let Ok((bar_entity, mut visual)) = bar_query.single_mut() else {
        return;
    };

    let shown_units = shown_status_units(hunger.current, hunger.max);
    let icon_count = status_icon_count(hunger.max);
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
            bar.spawn(status_icon_node(icons.hunger_icon(segment)));
        }
    });

    visual.shown_units = shown_units;
    visual.icon_count = icon_count;
}
