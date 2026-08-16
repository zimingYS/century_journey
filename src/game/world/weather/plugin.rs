//! 组装世界天气状态与固定步演化系统。

use crate::game::simulation::SimulationSet;
use crate::game::world::weather::cell::WeatherCell;
use crate::game::world::weather::systems::weather_evolve_system;
use bevy::prelude::*;

/// 组装天气状态资源与固定步演化系统。
///
/// 天气循环为负反馈（温度→湿度→云量→温度），在气候带约束下自稳定；
/// 马尔可夫状态只在游戏小时边界按领域随机流掷骰一次。
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherCell>().add_systems(
            FixedUpdate,
            weather_evolve_system.in_set(SimulationSet::Environment),
        );
    }
}
