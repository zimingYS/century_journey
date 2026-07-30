//! 根据窗口尺寸计算离散像素界面缩放比例。

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// 用户缩放和参考分辨率共同决定的界面缩放配置。
#[derive(Resource, Debug, Clone)]
pub struct UiScaleSettings {
    pub user_scale: f32,
    pub reference_size: Vec2,
    pub minimum_scale: f32,
    pub maximum_scale: f32,
}

impl Default for UiScaleSettings {
    fn default() -> Self {
        Self {
            user_scale: 1.0,
            reference_size: Vec2::new(1920.0, 1080.0),
            minimum_scale: 0.67,
            maximum_scale: 1.5,
        }
    }
}

impl UiScaleSettings {
    /// 计算指定视口下受上下限约束的最终界面缩放。
    pub fn resolved_scale(&self, viewport: Vec2) -> f32 {
        let fit = (viewport.x / self.reference_size.x)
            .min(viewport.y / self.reference_size.y)
            .clamp(self.minimum_scale, self.maximum_scale);
        fit * self.user_scale.clamp(0.5, 2.0)
    }
}

/// 窗口尺寸或用户配置变化时同步 Bevy 全局界面缩放。
pub fn sync_ui_scale_system(
    settings: Res<UiScaleSettings>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let target = settings.resolved_scale(Vec2::new(window.width(), window.height()));
    if (ui_scale.0 - target).abs() > f32::EPSILON {
        ui_scale.0 = target;
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/theme/scale.rs"]
mod tests;
