//! 客户端表现层。
//!
//! 本层把 Game 的状态转换为画面、声音、动画和界面，并采集本地输入。
//! 客户端可以产生动作请求，但不能直接拥有或复制游戏规则。
//!
//! 主要边界：
//! - Camera、Renderer、Sky：世界与相机渲染。
//! - Input、UI：输入上下文和用户界面。
//! - Effect、Particle、Sound：只消费状态与消息的反馈系统。
//! - Player：本地玩家模型及相机装配。

pub mod camera;
pub mod effect;
pub mod input;
pub mod particle;
pub mod player;
pub mod plugin_group;
pub mod renderer;
pub mod sky;
pub mod sound;
pub mod startup;
pub mod ui;
