//! 定义玩家战斗领域产生和消费的攻击消息。

use bevy::prelude::{Entity, Message};

/// 实体攻击请求，由战斗规则统一转换为伤害。
#[derive(Message, Debug, Clone, Copy)]
pub struct AttackEvent {
    /// 发起攻击的权威实体。
    pub attacker: Entity,
    /// 接收攻击的目标实体。
    pub target: Entity,
    /// 尚未经过生存防御规则结算的基础伤害。
    pub amount: f32,
}
