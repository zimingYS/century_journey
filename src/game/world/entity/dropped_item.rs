use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::CHUNK_SIZE;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

/// 掉落物默认存在时间，超过后自动销毁。
const DROPPED_ITEM_LIFETIME: f32 = 300.0;
/// 新掉落物在这段时间内不能被拾取，避免刚丢出就被玩家吸回。
const DROPPED_ITEM_PICKUP_DELAY: f32 = 0.85;
/// 掉落物与地面的简易碰撞半高。
const DROPPED_ITEM_GROUND_OFFSET: f32 = 0.06;
/// 掉落物重力加速度。
const DROPPED_ITEM_GRAVITY: f32 = 18.0;
/// 落地弹跳保留比例。
const DROPPED_ITEM_BOUNCE: f32 = 0.22;
/// 低于这个下落速度时不再弹起，直接进入落地状态。
const DROPPED_ITEM_REST_Y_SPEED: f32 = 0.9;
/// 空中水平速度阻尼。
const DROPPED_ITEM_AIR_DAMPING: f32 = 0.995;
/// 地面摩擦强度。
const DROPPED_ITEM_GROUND_FRICTION: f32 = 7.0;
/// 地面角速度阻尼。
const DROPPED_ITEM_ANGULAR_DAMPING: f32 = 5.5;
/// 速度足够小时认为物品已经停稳。
const DROPPED_ITEM_STOP_SPEED: f32 = 0.05;
/// 为方块破坏等被动掉落提供一点初始上抛速度，避免生成后死贴地面。
const PASSIVE_DROP_UP_SPEED: f32 = 1.1;
/// 玩家主动丢弃时的前抛速度。
const THROWN_DROP_FORWARD_SPEED: f32 = 4.2;
/// 玩家主动丢弃时的上抛速度。
const THROWN_DROP_UP_SPEED: f32 = 2.2;

/// 掉落物逻辑数据。
#[derive(Component, Debug, Clone)]
pub struct DroppedItem {
    /// 掉落物中保存的物品堆。
    pub stack: ItemStack,
    /// 已存在时间，超过生命周期后会销毁。
    pub age: f32,
    /// 拾取延迟，刚生成时避免立刻被玩家吸回。
    pub pickup_delay: f32,
    /// 是否已经与地面接触，用于控制摩擦和合并逻辑。
    pub grounded: bool,
}

impl DroppedItem {
    /// 创建一个新的掉落物组件。
    pub fn new(stack: ItemStack) -> Self {
        Self {
            stack,
            age: 0.0,
            pickup_delay: DROPPED_ITEM_PICKUP_DELAY,
            grounded: false,
        }
    }

    /// 判断当前掉落物是否允许被拾取。
    pub fn can_pickup(&self) -> bool {
        self.pickup_delay <= 0.0
    }
}

/// 掉落物简易刚体速度。
#[derive(Component, Debug, Clone, Copy)]
pub struct DroppedItemVelocity {
    /// 线速度，单位为方块/秒。
    pub linear: Vec3,
    /// 角速度，单位为弧度/秒。
    pub angular: Vec3,
}

impl DroppedItemVelocity {
    /// 创建静止速度。
    pub fn still() -> Self {
        Self {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        }
    }

    /// 方块破坏、掉落表等被动掉落使用的轻微弹出速度。
    pub fn passive(position: Vec3) -> Self {
        let jitter = Vec3::new(
            deterministic_wave(position.x + position.z) * 0.35,
            PASSIVE_DROP_UP_SPEED,
            deterministic_wave(position.z - position.x) * 0.35,
        );
        Self {
            linear: jitter,
            angular: Vec3::new(1.2, 0.8, -1.0),
        }
    }

    /// 玩家主动丢弃时使用的前抛速度。
    pub fn thrown(direction: Vec3) -> Self {
        let forward = horizontal_direction(direction);
        Self {
            linear: forward * THROWN_DROP_FORWARD_SPEED + Vec3::Y * THROWN_DROP_UP_SPEED,
            angular: Vec3::new(-forward.z, 0.35, forward.x) * 8.0,
        }
    }
}

impl Default for DroppedItemVelocity {
    fn default() -> Self {
        Self::still()
    }
}

/// 在指定的精确世界坐标生成一个带轻微弹出的掉落物实体。
pub fn spawn_dropped_item(commands: &mut Commands, position: Vec3, stack: ItemStack) -> Entity {
    let velocity = DroppedItemVelocity::passive(position);
    spawn_dropped_item_with_velocity(commands, position, stack, velocity)
}

/// 以指定世界坐标和速度生成掉落物。
///
/// 主动丢弃物品会走这个入口，这样物品不会额外叠加方块中心偏移。
pub fn spawn_dropped_item_with_velocity(
    commands: &mut Commands,
    position: Vec3,
    stack: ItemStack,
    velocity: DroppedItemVelocity,
) -> Entity {
    commands
        .spawn((
            DroppedItem::new(stack),
            velocity,
            Name::new("DroppedItem".to_string()),
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id()
}

/// 安静销毁掉落物实体以及它的视觉子实体。
///
/// 拾取、合并和生命周期系统可能在同一帧同时决定删除同一个掉落物，所以这里在应用命令时再次检查实体是否仍然存在。
pub fn despawn_dropped_item(commands: &mut Commands, entity: Entity) {
    commands.queue(move |world: &mut World| {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.despawn_related::<Children>();
            entity_mut.despawn();
        }
    });
}

/// 掉落物物理系统。
///
/// 模拟 ItemPhysicMod 风格的下落、弹跳、摩擦和滚动角速度；物品落地后不再无意义地竖直自转。
pub fn dropped_item_physics_system(
    time: Res<Time>,
    reg: Option<Res<BlockRegistry>>,
    storage: Res<WorldStorage>,
    mut query: Query<(&mut Transform, &mut DroppedItemVelocity, &mut DroppedItem)>,
) {
    let Some(reg) = reg.as_ref() else { return };
    let dt = time.delta_secs().min(0.05);

    for (mut transform, mut velocity, mut dropped) in &mut query {
        let was_grounded = dropped.grounded;
        if !was_grounded {
            velocity.linear.y -= DROPPED_ITEM_GRAVITY * dt;
        }

        velocity.linear.x *= DROPPED_ITEM_AIR_DAMPING;
        velocity.linear.z *= DROPPED_ITEM_AIR_DAMPING;

        let mut next_position = transform.translation + velocity.linear * dt;
        let mut should_be_grounded = false;

        if let Some(ground_y) = ground_height_at(next_position, &storage, reg)
            && crossed_ground_surface(transform.translation.y, next_position.y, ground_y)
        {
            next_position.y = ground_y;
            if velocity.linear.y < -DROPPED_ITEM_REST_Y_SPEED {
                velocity.linear.y = -velocity.linear.y * DROPPED_ITEM_BOUNCE;
                let bounce_spin = Vec3::new(velocity.linear.z, 0.0, -velocity.linear.x) * 0.35;
                velocity.angular += bounce_spin;
            } else {
                velocity.linear.y = 0.0;
                should_be_grounded = true;
            }

            let friction = (1.0 - DROPPED_ITEM_GROUND_FRICTION * dt).max(0.0);
            velocity.linear.x *= friction;
            velocity.linear.z *= friction;

            let angular_friction = (1.0 - DROPPED_ITEM_ANGULAR_DAMPING * dt).max(0.0);
            velocity.angular *= angular_friction;

            if Vec2::new(velocity.linear.x, velocity.linear.z).length() <= DROPPED_ITEM_STOP_SPEED {
                velocity.linear.x = 0.0;
                velocity.linear.z = 0.0;
            }
            if velocity.angular.length() <= DROPPED_ITEM_STOP_SPEED {
                velocity.angular = Vec3::ZERO;
            }
        }

        transform.translation = next_position;
        if velocity.angular.length_squared() > 0.0 {
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                velocity.angular.x * dt,
                velocity.angular.y * dt,
                velocity.angular.z * dt,
            ) * transform.rotation;
        }

        if should_be_grounded {
            // 落地后把父实体扶平，只保留水平朝向；子模型的 Ground 变换负责让工具和材料平躺在方块表面。
            transform.rotation = flattened_yaw_rotation(transform.rotation);
        }

        // 落地状态是掉落物自身数据，不再通过插拔组件更新，避免和同帧 despawn 命令冲突。
        dropped.grounded = should_be_grounded;
    }
}
/// 合并附近同类掉落物。
pub fn dropped_item_merge_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &DroppedItem)>,
) {
    let items: Vec<_> = query
        .iter()
        .filter(|(_, _, dropped)| dropped.grounded)
        .map(|(entity, transform, dropped)| (entity, transform.translation, dropped.stack.clone()))
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
            let total_count = items[i].2.count + items[j].2.count;
            let max_count = crate::game::inventory::item::stack::ItemStack::MAX_STACK_SIZE;
            if total_count <= max_count {
                let mut merged =
                    DroppedItem::new(ItemStack::new(items[i].2.item.clone(), total_count));
                merged.grounded = true;
                commands.entity(items[i].0).try_insert(merged);
                despawn_dropped_item(&mut commands, items[j].0);
                skip.insert(items[i].0);
                skip.insert(items[j].0);
            }
        }
    }
}
/// 更新掉落物生命周期和拾取延迟。
pub fn dropped_item_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DroppedItem)>,
) {
    for (entity, mut item) in &mut query {
        item.age += time.delta_secs();
        item.pickup_delay = (item.pickup_delay - time.delta_secs()).max(0.0);
        if item.age > DROPPED_ITEM_LIFETIME {
            despawn_dropped_item(&mut commands, entity);
        }
    }
}

/// 掉落物系统插件。
pub struct DroppedItemPlugin;
impl Plugin for DroppedItemPlugin {
    /// 只注册掉落物的玩法逻辑；视觉模型由 Client 渲染插件负责。
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dropped_item_physics_system,
                dropped_item_merge_system,
                dropped_item_tick_system,
            ),
        );
    }
}

/// 根据当前位置计算可站立地面的高度。
fn ground_height_at(pos: Vec3, storage: &WorldStorage, reg: &BlockRegistry) -> Option<f32> {
    let block_pos = IVec3::new(
        pos.x.floor() as i32,
        (pos.y - DROPPED_ITEM_GROUND_OFFSET).floor() as i32,
        pos.z.floor() as i32,
    );
    if solid_block_at(block_pos, storage, reg) {
        Some(block_pos.y as f32 + 1.0 + DROPPED_ITEM_GROUND_OFFSET)
    } else {
        None
    }
}

/// 只有物品从表面上方落下时才吸附到表面，防止嵌入树干后逐格向树顶传送。
fn crossed_ground_surface(previous_y: f32, next_y: f32, ground_y: f32) -> bool {
    previous_y >= ground_y && next_y <= ground_y
}

/// 判断指定世界方块坐标是否是实体方块。
fn solid_block_at(block_pos: IVec3, storage: &WorldStorage, reg: &BlockRegistry) -> bool {
    let chunk_pos = IVec3::new(
        block_pos.x.div_euclid(CHUNK_SIZE as i32),
        block_pos.y.div_euclid(CHUNK_SIZE as i32),
        block_pos.z.div_euclid(CHUNK_SIZE as i32),
    );
    let local_pos = IVec3::new(
        block_pos.x.rem_euclid(CHUNK_SIZE as i32),
        block_pos.y.rem_euclid(CHUNK_SIZE as i32),
        block_pos.z.rem_euclid(CHUNK_SIZE as i32),
    );

    storage.loaded_chunks.get(&chunk_pos).is_none_or(|chunk| {
        let id = chunk.get_voxel(
            local_pos.x as usize,
            local_pos.y as usize,
            local_pos.z as usize,
        );
        id != 0 && reg.get(id).is_some_and(|property| property.is_solid)
    })
}

/// 把任意姿态压平为只绕 Y 轴旋转的姿态。
fn flattened_yaw_rotation(rotation: Quat) -> Quat {
    let forward = rotation * Vec3::Z;
    let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z);
    if horizontal_forward.length_squared() > 0.0001 {
        let yaw = horizontal_forward.x.atan2(horizontal_forward.z);
        return Quat::from_rotation_y(yaw);
    }

    let right = rotation * Vec3::X;
    let horizontal_right = Vec3::new(right.x, 0.0, right.z);
    if horizontal_right.length_squared() > 0.0001 {
        let yaw = horizontal_right.x.atan2(horizontal_right.z) - std::f32::consts::FRAC_PI_2;
        Quat::from_rotation_y(yaw)
    } else {
        Quat::IDENTITY
    }
}

/// 把输入方向压成水平单位方向，避免玩家低头时把物品直接丢向地面。
fn horizontal_direction(direction: Vec3) -> Vec3 {
    let horizontal = Vec3::new(direction.x, 0.0, direction.z);
    if horizontal.length_squared() > 0.0001 {
        horizontal.normalize()
    } else {
        Vec3::Z
    }
}

/// 根据坐标生成确定性的轻微抖动，避免方块掉落物完全叠在一起。
fn deterministic_wave(value: f32) -> f32 {
    (value * 12.9898).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_fix_embedded_drop_is_not_lifted_to_block_top() {
        assert!(!crossed_ground_surface(4.5, 4.4, 5.06));
        assert!(crossed_ground_surface(5.2, 5.0, 5.06));
    }
}
