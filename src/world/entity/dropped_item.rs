use bevy::prelude::*;
use crate::inventory::item::stack::ItemStack;

/// 掉落物组件
#[derive(Component, Debug, Clone)]
pub struct DroppedItem {
    pub stack: ItemStack,
    /// 掉落物的存活时间（秒），超过后消失
    pub lifetime: f32,
}

impl DroppedItem {
    pub fn new(stack: ItemStack) -> Self {
        Self { stack, lifetime: 300.0 }
    }
}

/// 掉落物视觉标记 — 标记实体需要渲染
#[derive(Component)]
pub struct DroppedItemVisual;

/// 在指定世界坐标生成一个掉落物实体
pub fn spawn_dropped_item(commands: &mut Commands, position: Vec3, stack: ItemStack) -> Entity {
    commands
        .spawn((
            DroppedItem::new(stack),
            DroppedItemVisual,
            Name::new("DroppedItem".to_string()),
            Transform::from_translation(position + Vec3::new(0.5, 0.5, 0.5))
                .with_scale(Vec3::splat(0.25)),
            Visibility::default(),
        ))
        .id()
}

/// 为新生成的 DroppedItem 添加 3D 视觉（小的金色立方体）
pub fn dropped_item_visual_system(
    mut commands: Commands,
    new_items: Query<Entity, Added<DroppedItemVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.2),
        ..default()
    });

    for entity in &new_items {
        commands.entity(entity).insert((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
        ));
    }
}

/// 掉落物生命周期系统（超时销毁 + 悬浮动画 + 自旋）
pub fn dropped_item_lifecycle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DroppedItem, &mut Transform)>,
) {
    for (entity, mut item, mut transform) in &mut query {
        item.lifetime -= time.delta_secs();
        if item.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // 简单悬浮 + 旋转动画
        transform.rotate_y(time.delta_secs() * 2.0);
        let bounce = (time.elapsed_secs() * 3.0).sin() * 0.02;
        transform.translation.y += bounce;
    }
}

/// 掉落物插件
pub struct DroppedItemPlugin;

impl Plugin for DroppedItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            dropped_item_visual_system,
            dropped_item_lifecycle_system,
        ));
    }
}