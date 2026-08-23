//! 输入框历史回溯的单元测试。

use super::*;

#[test]
fn record_skips_blank_and_consecutive_duplicate_lines() {
    let mut history = InputHistory::default();
    history.record("  ");
    history.record("");
    assert!(history.entries.is_empty());

    history.record("/time set 600");
    history.record("  /time set 600  ");
    assert_eq!(history.entries, ["/time set 600"]);

    history.record("你好");
    assert_eq!(history.entries, ["/time set 600", "你好"]);
}

#[test]
fn record_drops_oldest_entries_beyond_the_cap() {
    let mut history = InputHistory::default();
    for index in 0..(MAX_ENTRIES + 2) {
        history.record(&format!("line-{index}"));
    }
    assert_eq!(history.entries.len(), MAX_ENTRIES);
    assert_eq!(history.entries.first().map(String::as_str), Some("line-2"));
    let expected_last = format!("line-{}", MAX_ENTRIES + 1);
    assert_eq!(
        history.entries.last().map(String::as_str),
        Some(expected_last.as_str())
    );
}

#[test]
fn browse_older_walks_toward_older_entries_and_clamps_at_the_oldest() {
    let mut history = InputHistory::default();
    history.record("first");
    history.record("second");
    history.record("third");

    assert_eq!(history.browse_older("draft"), Some("third"));
    assert_eq!(history.browse_older("draft"), Some("second"));
    assert_eq!(history.browse_older("draft"), Some("first"));
    // 已到最旧一条：停留并继续返回该条。
    assert_eq!(history.browse_older("draft"), Some("first"));
}

#[test]
fn browse_newer_walks_back_and_restores_the_saved_draft() {
    let mut history = InputHistory::default();
    history.record("first");
    history.record("second");

    assert_eq!(history.browse_older("typing"), Some("second"));
    assert_eq!(history.browse_older("typing"), Some("first"));
    assert_eq!(history.browse_newer(), Some("second"));
    // 越过最新一条：恢复进入浏览前保存的草稿。
    assert_eq!(history.browse_newer(), Some("typing"));
    // 回到草稿后继续按 ↓ 没有效果。
    assert_eq!(history.browse_newer(), None);
    // 从草稿再次进入浏览，重新保存当前草稿。
    assert_eq!(history.browse_older("retyped"), Some("second"));
    assert_eq!(history.browse_newer(), Some("retyped"));
}

#[test]
fn browsing_an_empty_history_does_nothing() {
    let mut history = InputHistory::default();
    assert_eq!(history.browse_older("draft"), None);
    assert_eq!(history.browse_newer(), None);
}

#[test]
fn reset_browsing_returns_to_a_fresh_draft() {
    let mut history = InputHistory::default();
    history.record("only");
    assert_eq!(history.browse_older("stale"), Some("only"));

    history.reset_browsing();
    // 重置后不在浏览状态：↓ 无效果。
    assert_eq!(history.browse_newer(), None);
    // 再次进入浏览保存的是新草稿，而不是重置前的旧草稿。
    assert_eq!(history.browse_older("fresh"), Some("only"));
    assert_eq!(history.browse_newer(), Some("fresh"));
}

#[test]
fn submitting_while_browsing_records_the_edited_line_and_resets() {
    let mut history = InputHistory::default();
    history.record("/time set 600");
    assert_eq!(history.browse_older(""), Some("/time set 600"));

    // 浏览状态下提交（可能已编辑）：记录提交内容并回到新行状态。
    history.record("/time set 1200");
    history.reset_browsing();
    assert_eq!(history.entries, ["/time set 600", "/time set 1200"]);
    assert_eq!(history.browse_newer(), None);
    assert_eq!(history.browse_older("next"), Some("/time set 1200"));
}
