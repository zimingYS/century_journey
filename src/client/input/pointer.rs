//! 追踪按钮指针状态变化，并发布与具体界面无关的交互生命周期消息。

use std::collections::HashMap;

use bevy::prelude::*;

/// 可交互 UI 元素相对于上一帧的指针阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiInteractionPhase {
    /// 指针刚进入元素。
    Hovered,
    /// 指针刚按下元素。
    Pressed,
    /// 指针持续按住元素。
    Held,
    /// 指针在元素上释放。
    Released,
    /// 指针离开或在元素外释放，导致交互取消。
    Cancelled,
}

/// UI 元素发生指针阶段变化时发出的客户端消息。
#[derive(Message, Debug, Clone, Copy)]
pub struct UiInteractionLifecycleEvent {
    /// 发生状态变化的 UI 实体。
    pub entity: Entity,
    /// 本帧解析出的指针阶段。
    pub phase: UiInteractionPhase,
}

/// 比较按钮前后帧状态并发布一次生命周期消息。
pub(super) fn ui_interaction_lifecycle_system(
    query: Query<(Entity, &Interaction), With<Button>>,
    mut previous: Local<HashMap<Entity, Interaction>>,
    mut writer: MessageWriter<UiInteractionLifecycleEvent>,
) {
    previous.retain(|entity, _| query.get(*entity).is_ok());
    for (entity, interaction) in &query {
        let old = previous.get(&entity).copied().unwrap_or(Interaction::None);
        if let Some(phase) = interaction_phase(old, *interaction) {
            writer.write(UiInteractionLifecycleEvent { entity, phase });
        }
        previous.insert(entity, *interaction);
    }
}

/// 将 Bevy 指针状态的前后变化归一化为客户端交互阶段。
pub(super) fn interaction_phase(
    previous: Interaction,
    current: Interaction,
) -> Option<UiInteractionPhase> {
    match (previous, current) {
        (Interaction::Pressed, Interaction::Pressed) => Some(UiInteractionPhase::Held),
        (_, Interaction::Pressed) => Some(UiInteractionPhase::Pressed),
        (Interaction::Pressed, Interaction::Hovered) => Some(UiInteractionPhase::Released),
        (Interaction::Pressed, Interaction::None) => Some(UiInteractionPhase::Cancelled),
        (Interaction::None, Interaction::Hovered) => Some(UiInteractionPhase::Hovered),
        (Interaction::Hovered, Interaction::None) => Some(UiInteractionPhase::Cancelled),
        _ => None,
    }
}
