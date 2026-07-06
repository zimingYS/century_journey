use crate::engine::asset::manager::AssetManager;
use bevy::prelude::*;

/// 手部渲染器
pub struct HandRenderer;

impl HandRenderer {
    pub fn build_hand_mesh(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::from_size(Vec3::new(0.15, 0.20, 0.08)))
    }

    /// 创建手部材质（通过 AssetManager）
    pub fn create_hand_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset: &mut AssetManager,
        asset_server: &AssetServer,
    ) -> Handle<StandardMaterial> {
        let id = crate::engine::asset::identifier::asset_id("textures/player/hand.png");
        let texture = asset.texture(&id, asset_server).handle;
        materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            base_color: Color::srgb(0.9, 0.75, 0.6),
            perceptual_roughness: 0.85,
            ..default()
        })
    }

    pub fn default_hand_transform() -> Transform {
        Transform {
            translation: Vec3::new(0.28, -0.25, -0.40),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                (-10_f32).to_radians(),
                0.0,
                (-5_f32).to_radians(),
            ),
            scale: Vec3::splat(0.55),
        }
    }
}
