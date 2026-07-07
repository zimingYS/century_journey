use crate::game::player::model::components::PlayerPart;
use crate::game::player::model::config::PlayerModelConfig;
use bevy::prelude::*;

/// 第一人称视角手部视图生成器。
///
/// 直接复用 PlayerModelConfig 的尺寸和配色，
/// 生成 forearm + hand 两段立方体——材质用纯色，不走贴图。
pub struct ViewHandBuilder;

impl ViewHandBuilder {
    /// 在 `parent`（通常为 ViewModelRoot）下生成手部实体层级：
    ///
    ///   parent
    ///     └── HandRoot
    ///           ├── Forearm (Mesh3d + MeshMaterial3d)
    ///           └── Hand    (Mesh3d + MeshMaterial3d) ← 返回此 entity
    ///
    /// 返回 `hand_entity`——手持物品应作为它的子节点放置。
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        config: &PlayerModelConfig,
        parent: Entity,
    ) -> Entity {
        // 前臂 Mesh
        let forearm_scale = config.scale(PlayerPart::forearm_r());
        let forearm_mesh = meshes.add(Cuboid::from_size(forearm_scale));
        let forearm_color = PlayerModelConfig::color(PlayerPart::forearm_r());
        let forearm_mat = materials.add(StandardMaterial {
            base_color: forearm_color,
            perceptual_roughness: 0.85,
            ..default()
        });

        // 手掌 Mesh
        let hand_scale = config.scale(PlayerPart::hand_r());
        let hand_mesh = meshes.add(Cuboid::from_size(hand_scale));
        let hand_color = PlayerModelConfig::color(PlayerPart::hand_r());
        let hand_mat = materials.add(StandardMaterial {
            base_color: hand_color,
            perceptual_roughness: 0.85,
            ..default()
        });

        // 根节点 (HandRoot)
        let root = commands
            .spawn((
                Name::new("ViewHandRoot"),
                Transform {
                    translation: Vec3::new(0.8, -0.5, -1.0),
                    rotation: Quat::from_euler(
                        EulerRot::XYZ,
                        120.0_f32.to_radians(),
                        0.0,
                        -15.0_f32.to_radians(),
                    ),
                    scale: Vec3::ONE,
                },
                Visibility::Inherited,
            ))
            .id();
        commands.entity(parent).add_child(root);

        let forearm_y = -forearm_scale.y * 0.5;
        let hand_y = -forearm_scale.y - hand_scale.y * 0.5;

        // 前臂
        let forearm = commands
            .spawn((
                Name::new("ViewForearm"),
                Mesh3d(forearm_mesh),
                MeshMaterial3d(forearm_mat),
                Transform::from_xyz(0.0, forearm_y, 0.0),
                // 掌心朝下，向上延伸
            ))
            .id();
        commands.entity(root).add_child(forearm);

        // 手掌
        let hand_entity = commands
            .spawn((
                Name::new("ViewHand"),
                Mesh3d(hand_mesh),
                MeshMaterial3d(hand_mat),
                Transform::from_xyz(0.0, hand_y, 0.0),
                // 位于前臂下方
            ))
            .id();
        commands.entity(root).add_child(hand_entity);

        // 返回手掌实体——物品挂在这个节点下
        hand_entity
    }
}
