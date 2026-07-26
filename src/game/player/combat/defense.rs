use bevy::prelude::Component;

/// 防御值
#[derive(Component, Debug, Clone, Default)]
pub struct Defense(pub f32);

impl Defense {
    pub fn damage_reduction(&self) -> f32 {
        if !self.0.is_finite() {
            return 0.0;
        }
        let defense = self.0.max(0.0);
        defense / (defense + 10.0)
    }
}
