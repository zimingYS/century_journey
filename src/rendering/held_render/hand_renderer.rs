use bevy::prelude::*;

/// 手部渲染器
pub struct HandRenderer;

impl HandRenderer {
    /// 构建右手网格
    pub fn build_hand_mesh(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::from_size(Vec3::new(0.15, 0.20, 0.08)))
    }

    /// 创建手部材质
    pub fn create_hand_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &AssetServer,
    ) -> Handle<StandardMaterial> {
        // 尝试加载手部皮肤，失败则用默认肤色
        let texture: Handle<Image> = asset_server.load("textures/player/hand.png");
        materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            base_color: Color::srgb(0.9, 0.75, 0.6),
            perceptual_roughness: 0.85,
            ..default()
        })
    }

    /// 默认空手
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
