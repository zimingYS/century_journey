use bevy::prelude::*;
use crate::game::player::model::components::{PlayerJoint, PlayerMesh, PlayerPart};

/// F7 切换骨架节点调试显示
/// 正式游玩这个系统应该集成到Debug内
pub fn debug_skeleton_system(
    input: Res<ButtonInput<KeyCode>>,
    mut show: Local<bool>,
    joint_query: Query<(&GlobalTransform, &PlayerJoint)>,
    mesh_query: Query<(&GlobalTransform, &PlayerMesh)>,
) {
    if input.just_pressed(KeyCode::F7) {
        *show = !*show;
        info!("[玩家调试] 骨架调试: {}", if *show { "ON" } else { "OFF" });
    }
    if !*show { return; }

    for (g_transform, joint) in &joint_query {
        info!("[关节] {:?}: ({:.2}, {:.2}, {:.2})", joint.0, g_transform.translation().x, g_transform.translation().y, g_transform.translation().z);
    }
    for (g_transform, mesh) in &mesh_query {
        info!("[纹理] {:?}: ({:.2}, {:.2}, {:.2})", mesh.0, g_transform.translation().x, g_transform.translation().y, g_transform.translation().z);
    }
    let _ = show;
}
