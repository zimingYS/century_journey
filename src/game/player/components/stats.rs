use bevy::prelude::*;

/// 生命值
#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
        }
    }
}

impl Health {
    pub fn fraction(&self) -> f32 {
        if !self.current.is_finite() || !self.max.is_finite() || self.max <= 0.0 {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
    pub fn apply_damage(&mut self, amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.current = (self.current - amount).max(0.0);
        }
    }
    pub fn apply_heal(&mut self, amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.current = (self.current + amount).min(self.max);
        }
    }
}

/// 饥饿值
#[derive(Component, Debug, Clone)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
    pub saturation: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
            saturation: 5.0,
        }
    }
}

impl Hunger {
    pub fn fraction(&self) -> f32 {
        if !self.current.is_finite() || !self.max.is_finite() || self.max <= 0.0 {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }
    pub fn is_starving(&self) -> bool {
        self.current <= 0.0
    }
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// 食用物品并恢复饥饿与饱和度。
    pub fn eat(&mut self, hunger: f32, saturation: f32) {
        if hunger.is_finite() && hunger > 0.0 {
            self.current = (self.current + hunger).min(self.max);
        }
        if saturation.is_finite() && saturation > 0.0 {
            self.saturation = (self.saturation + saturation).min(self.current.max(0.0));
        }
    }
    /// 消耗, 优先从 saturation 扣除
    pub fn exhaust(&mut self, amount: f32) {
        if !amount.is_finite() || amount <= 0.0 {
            return;
        }
        if self.saturation > 0.0 {
            let d = amount.min(self.saturation);
            self.saturation -= d;
            let rest = amount - d;
            self.current = (self.current - rest).max(0.0);
        } else {
            self.current = (self.current - amount).max(0.0);
        }
    }
}

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

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/components/stats.rs"]
mod tests;
