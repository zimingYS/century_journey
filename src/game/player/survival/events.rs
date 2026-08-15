//! 定义生存规则使用的伤害来源、反馈和食物消费消息。

use crate::shared::item_id::ItemId;
use bevy::prelude::{Entity, Message};

/// 受伤事件
#[derive(Message, Debug, Clone)]
pub struct DamageEvent {
    /// 承受伤害的玩家实体。
    pub target: Entity,
    /// 进入护甲计算前的原始伤害值。
    pub amount: f32,
    /// 用于规则和死亡提示的伤害来源。
    pub source: DamageSource,
}

/// 回血事件
#[derive(Message, Debug, Clone)]
pub struct HealEvent {
    /// 接受治疗的玩家实体。
    pub target: Entity,
    /// 请求恢复的生命值。
    pub amount: f32,
}

/// 食物已经实际消耗并恢复饥饿值。
#[derive(Message, Debug, Clone)]
pub struct FoodConsumedEvent {
    /// 实际消耗食物的玩家实体。
    pub player: Entity,
    /// 被消耗食物的稳定物品 ID。
    pub item: ItemId,
}

/// 饮品已经实际消耗并恢复口渴值。
#[derive(Message, Debug, Clone)]
pub struct DrinkConsumedEvent {
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
    /// 脱水
    Dehydration,
    /// 溺水
    Drowning,
    /// 火焰
    Fire,
    /// 过热
    Overheating,
    /// 失温
    Hypothermia,
    /// 实体
    Entity(Entity),
    /// 其他通用
    Generic,
}

impl DamageSource {
    /// 返回适合死亡提示使用的中文来源名称。
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Fall => "从高处坠落",
            Self::Starvation => "饥饿",
            Self::Dehydration => "脱水",
            Self::Drowning => "溺水",
            Self::Fire => "火焰",
            Self::Overheating => "过热",
            Self::Hypothermia => "失温",
            Self::Entity(_) => "实体攻击",
            Self::Generic => "环境伤害",
        }
    }
}
