use crate::shared::item_id::ItemId;
use bevy::prelude::{Entity, Message};

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

/// 食物已经实际消耗并恢复饥饿值。
#[derive(Message, Debug, Clone)]
pub struct FoodConsumedEvent {
    pub player: Entity,
    pub item: ItemId,
}

/// 受到伤害的来源
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl DamageSource {
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Fall => "从高处坠落",
            Self::Starvation => "饥饿",
            Self::Drowning => "溺水",
            Self::Fire => "火焰",
            Self::Entity(_) => "实体攻击",
            Self::Generic => "环境伤害",
        }
    }
}
