//! 文件日志：每次启动在 `logs/` 下按日期生成会话日志，并滚动清理历史。
//!
//! 与控制台输出并行写入同一份事件流；文件不带 ANSI 颜色、
//! 时间戳使用本地时区，便于玩家按日期检索。

use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy::log::{BoxedFmtLayer, BoxedLayer};
use chrono::{DateTime, Local, NaiveDate};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::registry::Registry;

/// 日志目录名，位于当前工作目录下。
pub const LOG_DIR: &str = "logs";
/// 历史会话日志的保留天数，超期文件在启动时删除。
pub const RETAIN_LOG_DAYS: i64 = 14;
/// 会话日志文件名格式：`game-日期_时间.log`。
const LOG_FILE_PATTERN: &str = "game-%Y-%m-%d_%H%M%S.log";

/// 本地时区时间戳，格式 `YYYY-MM-DD HH:MM:SS.mmm`。
struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

/// 由所有事件共享的追加式日志文件写入器。
#[derive(Clone)]
struct SharedFileWriter(Arc<Mutex<File>>);

impl io::Write for SharedFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lock_file()?.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.lock_file()?.flush()
    }
}

impl SharedFileWriter {
    fn lock_file(&self) -> io::Result<std::sync::MutexGuard<'_, File>> {
        // 锁中毒说明另一线程写入时 panic，文件状态已不可信，丢弃后续写入。
        self.0
            .lock()
            .map_err(|_| io::Error::other("日志文件锁已中毒"))
    }
}

impl<'a> MakeWriter<'a> for SharedFileWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self {
        self.clone()
    }
}

/// 生成会话日志文件路径；按启动时刻的本地日期与时间命名。
pub fn session_log_path(dir: &Path, now: DateTime<Local>) -> PathBuf {
    dir.join(now.format(LOG_FILE_PATTERN).to_string())
}

/// 判定日志文件是否早于保留截止日；无法解析的文件名不视为过期日志。
pub fn is_expired_log(filename: &str, today: NaiveDate, retain_days: i64) -> bool {
    let Some(date_part) = filename.strip_prefix("game-") else {
        return false;
    };
    let Some(date_part) = date_part.get(..10) else {
        return false;
    };
    match NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
        Ok(date) => date < today - chrono::Duration::days(retain_days),
        Err(_) => false,
    }
}

/// 删除目录下超过保留期的会话日志，返回删除数量；目录不可读时静默跳过。
pub fn prune_expired_logs(dir: &Path, now: DateTime<Local>) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let today = now.date_naive();
    let mut removed = 0;
    for entry in entries.flatten() {
        let Some(filename) = entry.file_name().into_string().ok() else {
            continue;
        };
        if is_expired_log(&filename, today, RETAIN_LOG_DAYS)
            && fs::remove_file(entry.path()).is_ok()
        {
            removed += 1;
        }
    }
    removed
}

/// 构造写入会话日志文件的输出层；初始化失败时返回 None，仅保留控制台输出。
pub fn session_log_layer() -> Option<BoxedLayer> {
    let dir = PathBuf::from(LOG_DIR);
    if let Err(error) = fs::create_dir_all(&dir) {
        eprintln!("[日志] 无法创建日志目录: {error}");
        return None;
    }
    let now = Local::now();
    let removed = prune_expired_logs(&dir, now);
    let path = session_log_path(&dir, now);
    let file = match File::options().create(true).append(true).open(&path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("[日志] 无法创建日志文件 {}: {error}", path.display());
            return None;
        }
    };
    if removed > 0 {
        eprintln!("[日志] 已清理 {removed} 个过期日志文件");
    }
    let writer = SharedFileWriter(Arc::new(Mutex::new(file)));
    let layer: Layer<Registry, _, _, _> = Layer::default()
        .with_ansi(false)
        .with_timer(LocalTimer)
        .with_writer(writer);
    Some(Box::new(layer))
}

/// 覆盖默认控制台输出层：保持 stderr 与彩色输出，仅把时间戳改为本地时区，
/// 使控制台与日志文件的时间一致。
pub fn console_fmt_layer() -> Option<BoxedFmtLayer> {
    Some(Box::new(
        Layer::default()
            .with_timer(LocalTimer)
            .with_writer(std::io::stderr),
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/app/application/logging.rs"]
mod tests;
