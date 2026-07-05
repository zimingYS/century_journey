use crate::client::ui::hud::bottom::bars::left_bars::armor_bar::{
    armor_bar_sync_system, spawn_armor_bar,
};
use crate::client::ui::hud::bottom::bars::left_bars::health_bar::{
    health_bar_sync_system, spawn_health_bar,
};
use crate::client::ui::hud::bottom::bars::right_bars::hunger_bar::{
    hunger_bar_sync_system, spawn_hunger_bar,
};
use crate::client::ui::hud::bottom::bars::spawn_bars_hud_system;
use crate::client::ui::hud::bottom::hotbar::{
    handle_hotbar_switch_system, hud_hotbar_visual_sync_system, spawn_hotbar_ui_system,
};
use crate::client::ui::hud::bottom::spawn_bottom_hud_system;
use crate::client::ui::hud::center::crosshair::spawn_crosshair;
use crate::client::ui::hud::center::spawn_center_hud_system;
use crate::client::ui::hud::left::spawn_left_hud_system;
use crate::client::ui::hud::left_bottom::spawn_left_bottom_hud_system;
use crate::client::ui::hud::left_top::spawn_left_top_hud_system;
use crate::client::ui::hud::right::spawn_right_hud_system;
use crate::client::ui::hud::right_bottom::spawn_right_bottom_hud_system;
use crate::client::ui::hud::right_top::spawn_right_top_hud_system;
use crate::client::ui::hud::spawn_hud_root_system;
use crate::client::ui::hud::top::spawn_top_hud_system;
use crate::shared::states::AppState;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HudSetupSet {
    Root,
    Anchor,
    Layout,
    Widget,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            OnEnter(AppState::InGame),
            (
                HudSetupSet::Root,
                HudSetupSet::Anchor,
                HudSetupSet::Layout,
                HudSetupSet::Widget,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_hud_root_system.in_set(HudSetupSet::Root),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            (
                spawn_bottom_hud_system,
                spawn_center_hud_system,
                spawn_left_hud_system,
                spawn_left_bottom_hud_system,
                spawn_left_top_hud_system,
                spawn_right_hud_system,
                spawn_right_bottom_hud_system,
                spawn_right_top_hud_system,
                spawn_top_hud_system,
            )
                .in_set(HudSetupSet::Anchor),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            (
                spawn_crosshair,
                spawn_bars_hud_system,
                spawn_hotbar_ui_system,
            )
                .in_set(HudSetupSet::Layout)
                .chain(),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            (spawn_health_bar, spawn_armor_bar).after(spawn_bars_hud_system),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            (spawn_hunger_bar,).after(spawn_bars_hud_system),
        );

        app.add_systems(
            Update,
            (
                hud_hotbar_visual_sync_system,
                handle_hotbar_switch_system,
                health_bar_sync_system,
                armor_bar_sync_system,
                hunger_bar_sync_system,
            ),
        );
    }
}
