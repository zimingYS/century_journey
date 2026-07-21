use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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
    pub fn resolved_scale(&self, viewport: Vec2) -> f32 {
        let fit = (viewport.x / self.reference_size.x)
            .min(viewport.y / self.reference_size.y)
            .clamp(self.minimum_scale, self.maximum_scale);
        fit * self.user_scale.clamp(0.5, 2.0)
    }
}

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
