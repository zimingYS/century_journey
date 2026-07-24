//! 带校验与备份恢复的原子文件写入。

use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 原子文件事务错误。
#[derive(Debug)]
pub struct AtomicFileError {
    message: String,
}

impl AtomicFileError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AtomicFileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AtomicFileError {}

/// 返回与主文件同目录的最近有效备份路径。
pub fn backup_path(path: &Path) -> PathBuf {
    append_suffix(path, ".bak")
}

/// 读取并校验主文件。
pub fn read_verified<F>(path: &Path, validate: F) -> Result<Vec<u8>, AtomicFileError>
where
    F: Fn(&[u8]) -> Result<(), String>,
{
    read_and_validate(path, &validate)
}

/// 读取并校验备份文件。
pub fn read_backup_verified<F>(path: &Path, validate: F) -> Result<Vec<u8>, AtomicFileError>
where
    F: Fn(&[u8]) -> Result<(), String>,
{
    read_and_validate(&backup_path(path), &validate)
}

/// 判断主文件是否拥有可恢复的有效备份。
pub fn has_valid_backup<F>(path: &Path, validate: F) -> bool
where
    F: Fn(&[u8]) -> Result<(), String>,
{
    read_and_validate(&backup_path(path), &validate).is_ok()
}

/// 将数据写入同目录临时文件，回读校验后替换主文件。
///
/// 如果旧主文件有效，会先将其移动为最近备份。进程在替换窗口被强制终止时，
/// 主文件或备份文件至少有一个仍保持有效。
pub fn atomic_write_verified<F>(
    path: &Path,
    bytes: &[u8],
    validate: F,
) -> Result<(), AtomicFileError>
where
    F: Fn(&[u8]) -> Result<(), String>,
{
    validate(bytes).map_err(|reason| {
        AtomicFileError::new(format!("拒绝写入无效数据 {}: {reason}", path.display()))
    })?;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        AtomicFileError::new(format!("创建目录 {} 失败: {error}", parent.display()))
    })?;

    let temporary = temporary_path(path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| {
                AtomicFileError::new(format!(
                    "创建临时文件 {} 失败: {error}",
                    temporary.display()
                ))
            })?;
        file.write_all(bytes).map_err(|error| {
            AtomicFileError::new(format!(
                "写入临时文件 {} 失败: {error}",
                temporary.display()
            ))
        })?;
        file.sync_all().map_err(|error| {
            AtomicFileError::new(format!(
                "同步临时文件 {} 失败: {error}",
                temporary.display()
            ))
        })?;
        drop(file);

        read_and_validate(&temporary, &validate)?;
        rotate_primary_to_backup(path, &validate)?;

        if let Err(error) = fs::rename(&temporary, path) {
            restore_primary_copy(path);
            return Err(AtomicFileError::new(format!(
                "替换主文件 {} 失败: {error}",
                path.display()
            )));
        }

        if let Err(error) = read_and_validate(path, &validate) {
            let _ = fs::remove_file(path);
            restore_primary_copy(path);
            return Err(error);
        }

        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// 使用最近有效备份恢复主文件，同时保留备份本身。
pub fn restore_backup<F>(path: &Path, validate: F) -> Result<(), AtomicFileError>
where
    F: Fn(&[u8]) -> Result<(), String>,
{
    let bytes = read_and_validate(&backup_path(path), &validate)?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| {
            AtomicFileError::new(format!("移除待恢复主文件 {} 失败: {error}", path.display()))
        })?;
    }
    atomic_write_verified(path, &bytes, validate)
}

fn rotate_primary_to_backup<F>(path: &Path, validate: &F) -> Result<(), AtomicFileError>
where
    F: Fn(&[u8]) -> Result<(), String>,
{
    if !path.exists() {
        return Ok(());
    }

    let primary_is_valid = read_and_validate(path, validate).is_ok();
    if !primary_is_valid {
        fs::remove_file(path).map_err(|error| {
            AtomicFileError::new(format!("移除损坏主文件 {} 失败: {error}", path.display()))
        })?;
        return Ok(());
    }

    let backup = backup_path(path);
    if backup.exists() {
        fs::remove_file(&backup).map_err(|error| {
            AtomicFileError::new(format!("轮换旧备份 {} 失败: {error}", backup.display()))
        })?;
    }
    fs::rename(path, &backup).map_err(|error| {
        AtomicFileError::new(format!(
            "创建最近有效备份 {} 失败: {error}",
            backup.display()
        ))
    })
}

fn restore_primary_copy(path: &Path) {
    let backup = backup_path(path);
    if !path.exists() && backup.exists() {
        let _ = fs::copy(backup, path);
    }
}

fn read_and_validate<F>(path: &Path, validate: &F) -> Result<Vec<u8>, AtomicFileError>
where
    F: Fn(&[u8]) -> Result<(), String>,
{
    let bytes = fs::read(path).map_err(|error| {
        AtomicFileError::new(format!("读取文件 {} 失败: {error}", path.display()))
    })?;
    validate(&bytes).map_err(|reason| {
        AtomicFileError::new(format!("校验文件 {} 失败: {reason}", path.display()))
    })?;
    Ok(bytes)
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(|| OsString::from("data"), OsString::from);
    name.push(suffix);
    path.with_file_name(name)
}

fn temporary_path(path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    append_suffix(path, &format!(".tmp-{}-{timestamp}", std::process::id()))
}

#[cfg(test)]
#[path = "../../tests/unit/engine/persistence.rs"]
mod tests;
