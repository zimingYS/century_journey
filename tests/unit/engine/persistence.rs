use super::*;

fn validate_text(bytes: &[u8]) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    if text.starts_with("valid:") {
        Ok(())
    } else {
        Err("缺少有效标记".into())
    }
}

fn test_directory(name: &str) -> PathBuf {
    let unique = temporary_path(Path::new(name));
    std::env::temp_dir().join(unique.file_name().unwrap_or_default())
}

#[test]
fn atomic_write_keeps_previous_valid_version_as_backup() {
    let directory = test_directory("atomic-backup");
    let path = directory.join("data.bin");
    atomic_write_verified(&path, b"valid:old", validate_text).unwrap();
    atomic_write_verified(&path, b"valid:new", validate_text).unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"valid:new");
    assert_eq!(fs::read(backup_path(&path)).unwrap(), b"valid:old");
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn interrupted_replace_can_restore_the_last_valid_file() {
    let directory = test_directory("atomic-interruption");
    let path = directory.join("data.bin");
    atomic_write_verified(&path, b"valid:stable", validate_text).unwrap();

    fs::rename(&path, backup_path(&path)).unwrap();
    assert!(!path.exists());
    assert!(has_valid_backup(&path, validate_text));

    restore_backup(&path, validate_text).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"valid:stable");
    assert_eq!(fs::read(backup_path(&path)).unwrap(), b"valid:stable");
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn explicit_restore_keeps_the_backup_available() {
    let directory = test_directory("atomic-explicit-restore");
    let path = directory.join("data.bin");
    atomic_write_verified(&path, b"valid:old", validate_text).unwrap();
    atomic_write_verified(&path, b"valid:new", validate_text).unwrap();

    restore_backup(&path, validate_text).unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"valid:old");
    assert_eq!(fs::read(backup_path(&path)).unwrap(), b"valid:old");
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn invalid_staged_data_never_replaces_primary() {
    let directory = test_directory("atomic-invalid");
    let path = directory.join("data.bin");
    atomic_write_verified(&path, b"valid:stable", validate_text).unwrap();

    assert!(atomic_write_verified(&path, b"broken", validate_text).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"valid:stable");
    let _ = fs::remove_dir_all(directory);
}
