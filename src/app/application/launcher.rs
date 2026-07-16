use super::application::Application;
use super::client::ClientApplication;
use super::editor::EditorApplication;
use super::mode::AppMode;
use super::server::ServerApplication;
use crate::app::config::AppConfig;

/// 程序入口。根据 AppConfig.mode 启动对应的 Application。
pub fn launch() -> anyhow::Result<()> {
    ensure_asset_working_directory()?;
    let config = AppConfig::default();
    match config.mode {
        AppMode::Client => ClientApplication::run(config),
        AppMode::Server => ServerApplication::run(config),
        AppMode::Editor => EditorApplication::run(config),
    }
}

fn ensure_asset_working_directory() -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    if has_required_assets(&current_dir) {
        return Ok(());
    }

    let executable = std::env::current_exe()?;
    if let Some(asset_root) = executable
        .ancestors()
        .skip(1)
        .take(5)
        .find(|candidate| has_required_assets(candidate))
    {
        std::env::set_current_dir(asset_root)?;
    }
    Ok(())
}

fn has_required_assets(root: &std::path::Path) -> bool {
    root.join("assets")
        .join("definitions")
        .join("blocks")
        .join("air.json")
        .is_file()
}
