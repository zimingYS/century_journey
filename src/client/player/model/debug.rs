//! 为玩家骨架关节和网格提供仅调试构建启用的可视化辅助。

use crate::app::settings::{KeyAction, Keybinds};
use crate::client::player::model::components::{PlayerJoint, PlayerMesh};
use bevy::prelude::*;

/// 骨架调试键切换骨架节点调试显示
/// 正式游玩这个系统应该集成到Debug内
pub fn debug_skeleton_system(
    input: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keybinds: Res<Keybinds>,
    context: Res<crate::shared::states::InputContextState>,
    mut show: Local<bool>,
    joint_query: Query<(&GlobalTransform, &PlayerJoint)>,
    mesh_query: Query<(&GlobalTransform, &PlayerMesh)>,
) {
    if context.active().allows_gameplay()
        && keybinds.is_just_pressed(KeyAction::ToggleSkeletonDebug, &input, &mouse)
    {
        *show = !*show;
        info!("[玩家调试] 骨架调试: {}", if *show { "ON" } else { "OFF" });
    }
    if !*show {
        return;
    }

    for (g_transform, joint) in &joint_query {
        info!(
            "[关节] {:?}: ({:.2}, {:.2}, {:.2})",
            joint.0,
            g_transform.translation().x,
            g_transform.translation().y,
            g_transform.translation().z
        );
    }
    for (g_transform, mesh) in &mesh_query {
        info!(
            "[纹理] {:?}: ({:.2}, {:.2}, {:.2})",
            mesh.0,
            g_transform.translation().x,
            g_transform.translation().y,
            g_transform.translation().z
        );
    }
    let _ = show;
}
