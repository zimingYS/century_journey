//! 管理客户端相机组件、视角切换、鼠标观察和渲染参数。

mod plugin;
mod systems;
mod types;

pub use plugin::CameraPlugin;
pub use types::{CameraPerspective, FpsCamera, MAX_CAMERA_PITCH, MIN_CAMERA_PITCH};

#[cfg(test)]
#[path = "../../../tests/unit/client/camera/mod.rs"]
mod tests;
