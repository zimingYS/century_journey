//! 文件日志模块的单元测试。

use super::*;
use chrono::NaiveDate;

#[test]
fn session_log_path_contains_local_date_and_time() {
    let dir = Path::new("logs");
    let now = Local::now();
    let path = session_log_path(dir, now);
    let filename = path.file_name().unwrap().to_str().unwrap().to_owned();
    assert!(filename.starts_with("game-"));
    assert!(filename.ends_with(".log"));
    // 文件名日期与传入时刻一致，按 game-YYYY-MM-DD_HHMMSS.log 解析回日期。
    let date_part = &filename["game-".len().."game-".len() + 10];
    let parsed = NaiveDate::parse_from_str(date_part, "%Y-%m-%d").unwrap();
    assert_eq!(parsed, now.date_naive());
}

#[test]
fn is_expired_log_marks_only_old_log_files() {
    let today = NaiveDate::from_ymd_opt(2026, 8, 24).unwrap();
    // 刚好保留期边界：14 天前的日期是最后一天保留日。
    let boundary = today - chrono::Duration::days(RETAIN_LOG_DAYS);
    let expired = boundary.pred_opt().unwrap();
    assert!(!is_expired_log(
        &format!("game-{}.log", boundary.format("%Y-%m-%d")),
        today,
        RETAIN_LOG_DAYS
    ));
    assert!(is_expired_log(
        &format!("game-{}.log", expired.format("%Y-%m-%d")),
        today,
        RETAIN_LOG_DAYS
    ));
    assert!(!is_expired_log(
        &format!("game-{}.log", today.format("%Y-%m-%d")),
        today,
        RETAIN_LOG_DAYS
    ));
}

#[test]
fn is_expired_log_ignores_unrelated_files() {
    let today = NaiveDate::from_ymd_opt(2026, 8, 24).unwrap();
    let old = today - chrono::Duration::days(RETAIN_LOG_DAYS * 2);
    // 前缀不符、日期无法解析的文件都不动。
    assert!(!is_expired_log("notes.txt", today, RETAIN_LOG_DAYS));
    assert!(!is_expired_log(
        &format!("game-{}.log", old.format("%Y/%m/%d")),
        today,
        RETAIN_LOG_DAYS
    ));
    assert!(!is_expired_log("game-broken.log", today, RETAIN_LOG_DAYS));
}

#[test]
fn prune_expired_logs_removes_only_expired_files() {
    let dir = std::env::temp_dir().join("cj_logging_test_prune");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let today = Local::now();
    let expired_name = format!(
        "game-{}.log",
        (today.date_naive() - chrono::Duration::days(RETAIN_LOG_DAYS + 3)).format("%Y-%m-%d")
    );
    let fresh_name = format!("game-{}.log", today.date_naive().format("%Y-%m-%d"));
    std::fs::write(dir.join(&expired_name), "old").unwrap();
    std::fs::write(dir.join(&fresh_name), "new").unwrap();
    std::fs::write(dir.join("unrelated.txt"), "keep").unwrap();

    let removed = prune_expired_logs(&dir, today);
    assert_eq!(removed, 1);
    assert!(!dir.join(&expired_name).exists());
    assert!(dir.join(&fresh_name).exists());
    assert!(dir.join("unrelated.txt").exists());

    let _ = std::fs::remove_dir_all(&dir);
}
