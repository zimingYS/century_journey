use bevy::prelude::{Entity, Message};

/// 实体攻击请求，由战斗规则统一转换为伤害。
#[derive(Message, Debug, Clone, Copy)]
pub struct AttackEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub amount: f32,
}
