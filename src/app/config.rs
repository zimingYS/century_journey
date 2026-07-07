//! # Config
//!
//! 应用配置。
//!
//! 定义全局配置项，并负责配置的加载、管理与访问。

use crate::app::application::mode::AppMode;
use std::path::PathBuf;

/// 应用配置。
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 应用运行模式。
    pub mode: AppMode,
    /// 窗口配置。
    pub window: WindowConfig,
    /// 渲染配置。
    pub render: RenderConfig,
    /// 网络配置。
    pub network: NetworkConfig,
    /// 存档配置。
    pub save: SaveConfig,
    /// 日志配置。
    pub logging: LoggingConfig,
    /// 调试配置。
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

/// 窗口配置
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// 标题
    pub title: String,
    /// 分辨率
    pub width: u32,
    pub height: u32,
    /// 全屏模式
    pub fullscreen: bool,
    /// 垂直同步
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

/// 渲染配置
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// 渲染距离
    pub render_distance: u32,
    /// 阴影距离
    pub shadow_distance: u32,
    /// 多重采样抗锯齿
    pub msaa: bool,
    /// 高动态光照渲染
    pub hdr: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            render_distance: 16,
            shadow_distance: 8,
            msaa: true,
            hdr: false,
        }
    }
}

/// 网络配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// 监听地址
    pub address: String,
    /// 监听端口
    pub port: u16,
    /// 连接超时时间(秒)
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

/// 存档配置
#[derive(Debug, Clone)]
pub struct SaveConfig {
    /// 存档目录
    pub world_directory: PathBuf,
    /// 自动保存间隔秒数
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

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// 输出日志等级
    pub level: LogLevel,
    /// 是否保存为文件
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

/// 日志等级
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// 调试配置
#[derive(Debug, Clone, Default)]
pub struct DebugConfig {
    /// 是否开启诊断
    pub diagnostics: bool,
    /// 是否开启实体线框
    pub wireframe: bool,
    /// 是否开启分析器
    pub profiler: bool,
}
