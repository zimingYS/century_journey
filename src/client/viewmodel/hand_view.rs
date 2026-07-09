use crate::client::viewmodel::ViewModelPart;
use crate::game::player::model::components::PlayerPart;
use crate::game::player::model::config::PlayerModelConfig;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

pub struct ViewHandBuilder;

impl ViewHandBuilder {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        config: &PlayerModelConfig,
        parent: Entity,
    ) -> Entity {
        let forearm_scale = config.scale(PlayerPart::forearm_r());
        let forearm_mesh = meshes.add(Cuboid::from_size(forearm_scale));
        let forearm_mat = materials.add(StandardMaterial {
            base_color: PlayerModelConfig::color(PlayerPart::forearm_r()),
            perceptual_roughness: 0.9,
            ..default()
        });

        let hand_scale = config.scale(PlayerPart::hand_r());
        let hand_mesh = meshes.add(Cuboid::from_size(hand_scale));
        let hand_mat = materials.add(StandardMaterial {
            base_color: PlayerModelConfig::color(PlayerPart::hand_r()),
            perceptual_roughness: 0.9,
            ..default()
        });

        let root = commands
            .spawn((
                Name::new("ViewHandRoot"),
                ViewModelPart,
                Transform {
                    translation: Vec3::new(0.8, -0.58, -0.92),
                    rotation: Quat::from_euler(
                        EulerRot::XYZ,
                        103.0_f32.to_radians(),
                        -5.0_f32.to_radians(),
                        -18.0_f32.to_radians(),
                    ),
                    scale: Vec3::splat(0.78),
                },
                Visibility::Inherited,
            ))
            .id();
        commands.entity(parent).add_child(root);

        let item_anchor = commands
            .spawn((
                Name::new("ViewHeldItemAnchor"),
                ViewModelPart,
                Transform::from_xyz(0.8, -0.52, -1.02),
                Visibility::Inherited,
            ))
            .id();
        commands.entity(parent).add_child(item_anchor);

        let forearm_y = -forearm_scale.y * 0.5;
        let hand_y = -forearm_scale.y - hand_scale.y * 0.5;

        let forearm = commands
            .spawn((
                Name::new("ViewForearm"),
                ViewModelPart,
                Mesh3d(forearm_mesh),
                MeshMaterial3d(forearm_mat),
                Transform::from_xyz(0.0, forearm_y, 0.0),
                Visibility::Inherited,
                NotShadowCaster,
            ))
            .id();
        commands.entity(root).add_child(forearm);

        let hand_entity = commands
            .spawn((
                Name::new("ViewHand"),
                ViewModelPart,
                Mesh3d(hand_mesh),
                MeshMaterial3d(hand_mat),
                Transform::from_xyz(0.0, hand_y, 0.0),
                Visibility::Inherited,
                NotShadowCaster,
            ))
            .id();
        commands.entity(root).add_child(hand_entity);

        item_anchor
    }
}
