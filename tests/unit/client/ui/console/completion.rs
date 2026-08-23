//! Tab 补全循环选择的单元测试。

use super::*;

fn candidates() -> Vec<String> {
    vec!["/time set ".to_owned(), "/time scale ".to_owned()]
}

#[test]
fn first_tab_selects_the_first_candidate() {
    let mut tracker = CompletionTracker::default();
    assert_eq!(
        tracker.choose("/time s", &candidates()),
        Some("/time set ".to_owned())
    );
}

#[test]
fn repeated_tabs_cycle_through_candidates() {
    let mut tracker = CompletionTracker::default();
    assert_eq!(
        tracker.choose("/time s", &candidates()),
        Some("/time set ".to_owned())
    );
    assert_eq!(
        tracker.choose("/time set ", &candidates()),
        Some("/time scale ".to_owned())
    );
    assert_eq!(
        tracker.choose("/time scale ", &candidates()),
        Some("/time set ".to_owned())
    );
}

#[test]
fn editing_between_tabs_restarts_from_the_first_candidate() {
    let mut tracker = CompletionTracker::default();
    tracker.choose("/time s", &candidates());
    assert_eq!(
        tracker.choose("/time set 6", &candidates()),
        Some("/time set ".to_owned())
    );
}

#[test]
fn empty_candidates_reset_state_and_return_none() {
    let mut tracker = CompletionTracker::default();
    tracker.choose("/time s", &candidates());
    assert_eq!(tracker.choose("/time set ", &[]), None);
    // 重置后再次补全从第一个候选开始。
    assert_eq!(
        tracker.choose("/time s", &candidates()),
        Some("/time set ".to_owned())
    );
}

#[test]
fn reset_clears_the_anchor_so_tabs_start_over() {
    let mut tracker = CompletionTracker::default();
    tracker.choose("/time s", &candidates());
    tracker.reset();
    assert_eq!(
        tracker.choose("/time set ", &candidates()),
        Some("/time set ".to_owned())
    );
}
