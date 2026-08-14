//! 根据玩家口渴值更新水滴图标和空心状态。

use crate::client::ui::hud::bottom::bars::{
    HUD_STATUS_ICON_GAP, HudStatusIconAssets, RightBarsHud, shown_status_units, status_icon_count,
    status_icon_node, status_icon_segment,
};
use crate::game::player::identity::Player;
use crate::game::player::survival::thirst::Thirst;
use bevy::prelude::*;

/// 口渴值条根节点。
#[derive(Component)]
pub struct ThirstBar;

/// 口渴值条当前已经绘制的状态。
#[derive(Component, Default, PartialEq, Eq)]
pub struct ThirstBarVisual {
    shown_units: u32,
    icon_count: usize,
}

/// 生成口渴值 HUD。
pub fn spawn_thirst_bar(mut commands: Commands, bars_hud: Query<Entity, With<RightBarsHud>>) {
    let Ok(bars_hud_entity) = bars_hud.single() else {
        log::error!("BARS HUD NOT FOUND - cannot spawn thirst bar");
        return;
    };

    commands.entity(bars_hud_entity).with_children(|parent| {
        parent.spawn((
            ThirstBar,
            ThirstBarVisual::default(),
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(HUD_STATUS_ICON_GAP),
                ..default()
            },
        ));
    });
}

/// 根据口渴值同步 HUD 图标显示。
pub fn thirst_bar_sync_system(
    thirst_query: Query<&Thirst, With<Player>>,
    mut bar_query: Query<(Entity, &mut ThirstBarVisual), With<ThirstBar>>,
    children_query: Query<&Children>,
    icons: Res<HudStatusIconAssets>,
    mut commands: Commands,
) {
    let Ok(thirst) = thirst_query.single() else {
        return;
    };
    let Ok((bar_entity, mut visual)) = bar_query.single_mut() else {
        return;
    };

    let shown_units = shown_status_units(thirst.current, thirst.max);
    let icon_count = status_icon_count(thirst.max);
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
            let logical_index = icon_count - 1 - index;
            let segment = status_icon_segment(shown_units, logical_index);
            bar.spawn(status_icon_node(icons.thirst_icon(segment)));
        }
    });

    visual.shown_units = shown_units;
    visual.icon_count = icon_count;
}
