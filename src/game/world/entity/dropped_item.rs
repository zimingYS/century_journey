use crate::content::block::registry::BlockRegistry;
use crate::engine::constant::world::CHUNK_SIZE;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct DroppedItem {
    pub stack: ItemStack,
    pub age: f32,
    pub pickup_delay: f32,
}

impl DroppedItem {
    pub fn new(stack: ItemStack) -> Self {
        Self {
            stack,
            age: 0.0,
            pickup_delay: 0.5,
        }
    }
    pub fn can_pickup(&self) -> bool {
        self.pickup_delay <= 0.0
    }
}

#[derive(Component, Default)]
pub struct DroppedItemVelocity {
    pub y: f32,
}

#[derive(Component)]
pub struct DroppedItemVisual;

#[derive(Component, Default)]
pub struct DroppedItemGrounded;

fn solid_below(pos: Vec3, storage: &WorldStorage, reg: &BlockRegistry) -> bool {
    let bx = pos.x.floor() as i32;
    let by = (pos.y - 0.3).floor() as i32;
    let bz = pos.z.floor() as i32;
    let cp = IVec3::new(
        bx.div_euclid(CHUNK_SIZE as i32),
        by.div_euclid(CHUNK_SIZE as i32),
        bz.div_euclid(CHUNK_SIZE as i32),
    );
    let lp = IVec3::new(
        bx.rem_euclid(CHUNK_SIZE as i32),
        by.rem_euclid(CHUNK_SIZE as i32),
        bz.rem_euclid(CHUNK_SIZE as i32),
    );
    storage.loaded_chunks.get(&cp).map_or(true, |c| {
        let id = c.get_voxel(lp.x as usize, lp.y as usize, lp.z as usize);
        id != 0 && reg.get(id).map_or(false, |p| p.is_solid)
    })
}

pub fn spawn_dropped_item(commands: &mut Commands, position: Vec3, stack: ItemStack) -> Entity {
    commands
        .spawn((
            DroppedItem::new(stack),
            DroppedItemVisual,
            DroppedItemVelocity::default(),
            Name::new("DroppedItem".to_string()),
            Transform::from_translation(position + Vec3::new(0.5, 1.0, 0.5))
                .with_scale(Vec3::splat(0.25)),
            Visibility::default(),
        ))
        .id()
}

pub fn dropped_item_visual_system(
    mut commands: Commands,
    new: Query<Entity, Added<DroppedItemVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let m = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat = mats.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.2),
        ..default()
    });
    for e in &new {
        commands
            .entity(e)
            .insert((Mesh3d(m.clone()), MeshMaterial3d(mat.clone())));
    }
}

/// 重力
pub fn dropped_item_gravity_system(
    time: Res<Time>,
    reg: Option<Res<BlockRegistry>>,
    storage: Res<WorldStorage>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut DroppedItemVelocity), Without<DroppedItemGrounded>>,
) {
    let Some(reg) = reg.as_ref() else { return };
    for (e, t, mut v) in &mut query {
        v.y -= 15.0 * time.delta_secs();
        let ny = t.translation.y + v.y * time.delta_secs();
        if solid_below(
            Vec3::new(t.translation.x, ny, t.translation.z),
            &storage,
            reg,
        ) {
            let ground_y = (ny - 0.3).floor() as f32 + 1.0 + 0.3;
            commands.entity(e).insert((
                Transform::from_translation(Vec3::new(t.translation.x, ground_y, t.translation.z))
                    .with_scale(Vec3::splat(0.25)),
                DroppedItemGrounded,
            ));
        } else {
            commands.entity(e).insert(
                Transform::from_translation(Vec3::new(t.translation.x, ny, t.translation.z))
                    .with_scale(Vec3::splat(0.25)),
            );
        }
    }
}

/// 同种合并 (2m内)
pub fn dropped_item_merge_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &DroppedItem), With<DroppedItemGrounded>>,
) {
    let items: Vec<_> = query
        .iter()
        .map(|(e, t, d)| (e, t.translation, d.stack.clone()))
        .collect();
    let mut skip = std::collections::HashSet::new();
    for i in 0..items.len() {
        if skip.contains(&items[i].0) {
            continue;
        }
        for j in (i + 1)..items.len() {
            if skip.contains(&items[j].0) {
                continue;
            }
            if items[i].1.distance(items[j].1) > 1.5 {
                continue;
            }
            if items[i].2.item != items[j].2.item {
                continue;
            }
            let tot = items[i].2.count + items[j].2.count;
            let max = crate::game::inventory::item::stack::ItemStack::MAX_STACK_SIZE;
            if tot <= max {
                commands
                    .entity(items[i].0)
                    .insert(DroppedItem::new(ItemStack::new(
                        items[i].2.item.clone(),
                        tot,
                    )));
                commands.entity(items[j].0).despawn();
                skip.insert(items[i].0);
                skip.insert(items[j].0);
            }
        }
    }
}

/// 超时销毁 + pickup delay 递减
pub fn dropped_item_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DroppedItem)>,
) {
    for (e, mut item) in &mut query {
        item.age += time.delta_secs();
        item.pickup_delay = (item.pickup_delay - time.delta_secs()).max(0.0);
        if item.age > 300.0 {
            commands.entity(e).despawn();
        }
    }
}

/// 自旋
pub fn dropped_item_spin_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DroppedItemGrounded>>,
) {
    for mut t in &mut query {
        t.rotate_y(time.delta_secs() * 2.0);
    }
}

pub struct DroppedItemPlugin;
impl Plugin for DroppedItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dropped_item_visual_system,
                dropped_item_gravity_system,
                dropped_item_merge_system,
                dropped_item_tick_system,
                dropped_item_spin_system,
            ),
        );
    }
}
