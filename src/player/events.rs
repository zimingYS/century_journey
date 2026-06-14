use bevy::prelude::*;

/// 受伤事件
#[derive(Message, Debug, Clone)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: DamageSource,
}

/// 回血事件
#[derive(Message, Debug, Clone)]
pub struct HealEvent {
    pub target: Entity,
    pub amount: f32,
}

/// 死亡事件
#[derive(Message, Debug, Clone)]
pub struct DeathEvent {
    pub entity: Entity,
}

/// 受到伤害的来源
#[derive(Debug, Clone, Copy)]
pub enum DamageSource {
    /// 摔落
    Fall,
    /// 饥饿
    Starvation,
    /// 溺水
    Drowning,
    /// 火焰
    Fire,
    /// 实体
    Entity(Entity),
    /// 其他通用
    Generic,
}