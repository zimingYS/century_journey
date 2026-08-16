//! 组装降水粒子发射与更新系统。

use bevy::prelude::*;

use crate::client::weather::systems::{spawn_precipitation_system, update_precipitation_system};
use crate::client::weather::types::PrecipitationVisuals;
use crate::shared::states::AppState;

/// 注册降水粒子发射与更新系统。
pub struct ClientWeatherPlugin;

impl Plugin for ClientWeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrecipitationVisuals>().add_systems(
            Update,
            (spawn_precipitation_system, update_precipitation_system)
                .chain()
                .run_if(in_state(AppState::InGame)),
        );
    }
}
