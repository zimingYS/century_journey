//! Application configuration.

use crate::app::application::mode::AppMode;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mode: AppMode,
    pub window: WindowConfig,
    pub render: RenderConfig,
    pub network: NetworkConfig,
    pub save: SaveConfig,
    pub logging: LoggingConfig,
    pub debug: DebugConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: AppMode::Client,
            window: WindowConfig::default(),
            render: RenderConfig::default(),
            network: NetworkConfig::default(),
            save: SaveConfig::default(),
            logging: LoggingConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Century Journey".into(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub render_distance: u32,
    pub mesh_distance: u32,
    pub data_vertical_radius_above: u32,
    pub data_vertical_radius_below: u32,
    pub shadow_distance: u32,
    pub msaa: bool,
    pub hdr: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            render_distance: 8,
            mesh_distance: 8,
            data_vertical_radius_above: 2,
            data_vertical_radius_below: 3,
            shadow_distance: 8,
            msaa: true,
            hdr: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub address: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".into(),
            port: 28885,
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveConfig {
    pub world_directory: PathBuf,
    pub autosave_interval_seconds: u64,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            world_directory: PathBuf::from("saves"),
            autosave_interval_seconds: 300,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file_output: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_output: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct DebugConfig {
    pub diagnostics: bool,
    pub wireframe: bool,
    pub profiler: bool,
}
