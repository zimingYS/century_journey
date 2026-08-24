//! Toast 排队逻辑的单元测试。

use super::*;
use crate::game::notification::{NotificationLevel, PlayerNotification};

fn notification(text: &str) -> PlayerNotification {
    PlayerNotification {
        text: text.to_owned(),
        level: NotificationLevel::Info,
    }
}

#[test]
fn admit_respects_active_capacity() {
    let mut queue = ToastQueue::default();
    for index in 0..6 {
        queue.push(notification(&format!("通知{index}")));
    }
    let admitted = queue.admit(0);
    assert_eq!(admitted.len(), MAX_ACTIVE_TOASTS);
    assert_eq!(admitted[0].text, "通知0");
    assert_eq!(queue.pending.len(), 6 - MAX_ACTIVE_TOASTS);
}

#[test]
fn admit_releases_remaining_when_slots_free() {
    let mut queue = ToastQueue::default();
    for index in 0..6 {
        queue.push(notification(&format!("通知{index}")));
    }
    queue.admit(0);
    // 两条回收后，剩余两条排队通知按序放行。
    let admitted = queue.admit(MAX_ACTIVE_TOASTS - 2);
    assert_eq!(admitted.len(), 2);
    assert_eq!(admitted[0].text, "通知4");
    assert_eq!(admitted[1].text, "通知5");
    assert_eq!(queue.pending.len(), 0);
}

#[test]
fn admit_returns_empty_when_stack_full() {
    let mut queue = ToastQueue::default();
    queue.push(notification("通知"));
    assert!(queue.admit(MAX_ACTIVE_TOASTS).is_empty());
    assert_eq!(queue.pending.len(), 1);
}

#[test]
fn pending_overflow_drops_oldest() {
    let mut queue = ToastQueue::default();
    for index in 0..(MAX_PENDING_TOASTS + 4) {
        queue.push(notification(&format!("通知{index}")));
    }
    assert_eq!(queue.pending.len(), MAX_PENDING_TOASTS);
    // 最旧的 4 条被丢弃，队首从第 5 条开始。
    let admitted = queue.admit(0);
    assert_eq!(admitted[0].text, "通知4");
}

#[test]
fn admit_on_empty_queue_returns_empty() {
    let mut queue = ToastQueue::default();
    assert!(queue.admit(0).is_empty());
    assert_eq!(queue.pending.len(), 0);
}
