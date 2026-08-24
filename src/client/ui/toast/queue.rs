//! 通知入队与展示准入的纯排队逻辑，不依赖 ECS 系统。

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::game::notification::PlayerNotification;

/// 同屏最多展示的 Toast 条数；超出部分在队列中等待空位。
pub const MAX_ACTIVE_TOASTS: usize = 4;

/// 排队等待的通知上限；超出时丢弃最旧通知，防止队列无限增长。
pub const MAX_PENDING_TOASTS: usize = 16;

/// 待展示通知的排队状态。
#[derive(Debug, Default, Resource)]
pub struct ToastQueue {
    pending: VecDeque<PlayerNotification>,
}

impl ToastQueue {
    /// 追加一条待展示通知；队列已满时丢弃最旧通知。
    pub fn push(&mut self, notification: PlayerNotification) {
        if self.pending.len() >= MAX_PENDING_TOASTS {
            self.pending.pop_front();
        }
        self.pending.push_back(notification);
    }

    /// 按剩余展示容量放行通知；返回本帧应生成的通知列表。
    ///
    /// active_count 为当前屏幕上尚未回收的 Toast 数。
    pub fn admit(&mut self, active_count: usize) -> Vec<PlayerNotification> {
        let capacity = MAX_ACTIVE_TOASTS.saturating_sub(active_count);
        let mut admitted = Vec::with_capacity(capacity.min(self.pending.len()));
        for _ in 0..capacity {
            match self.pending.pop_front() {
                Some(notification) => admitted.push(notification),
                None => break,
            }
        }
        admitted
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/toast/queue.rs"]
mod tests;
