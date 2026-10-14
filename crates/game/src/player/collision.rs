//! 玩家碰撞检测

use bevy::prelude::*;
use cj_core::pos::BlockPos;
use cj_core::spec::{WORLD_BOTTOM, WORLD_TOP};
use cj_runtime::world_data::WorldData;

/// AABB 碰撞轴。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

/// 玩家碰撞箱尺寸。
pub const PLAYER_SIZE: Vec3 = Vec3::new(0.6, 1.8, 0.6);

/// 玩家 AABB 半尺寸
pub fn player_half_extents() -> Vec3 {
    PLAYER_SIZE * 0.5
}

/// 检查指定 AABB 是否与任意实心方块发生碰撞
fn check_collision_at<F>(position: Vec3, half_extents: Vec3, is_solid: F) -> bool
where
    F: Fn(BlockPos) -> bool,
{
    let min = position - half_extents;
    let max = position + half_extents;

    let min_x = min.x.floor() as i32;
    let max_x = max.x.ceil() as i32 - 1;

    let min_y = min.y.floor() as i32;
    let max_y = max.y.ceil() as i32 - 1;

    let min_z = min.z.floor() as i32;
    let max_z = max.z.ceil() as i32 - 1;

    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let block = BlockPos::new(x, y, z);

                if !is_solid(block) {
                    continue;
                }

                // AABB 与方块 AABB 的精确重叠检查。
                if position.x - half_extents.x < (x + 1) as f32
                    && position.x + half_extents.x > x as f32
                    && position.y - half_extents.y < (y + 1) as f32
                    && position.y + half_extents.y > y as f32
                    && position.z - half_extents.z < (z + 1) as f32
                    && position.z + half_extents.z > z as f32
                {
                    return true;
                }
            }
        }
    }

    false
}

/// 世界层的方块实心判断。
fn world_is_solid(world: &WorldData, pos: BlockPos) -> bool {
    if pos.y < WORLD_BOTTOM || pos.y >= WORLD_TOP {
        return true;
    }

    let chunk_pos = pos.chunk_pos();

    let Some(chunk) = world.chunks().get(chunk_pos) else {
        return false;
    };

    chunk.get_block(pos).block().is_solid()
}

/// 沿单个轴移动。
fn move_axis<F>(
    position: Vec3,
    amount: f32,
    half_extents: Vec3,
    axis: Axis,
    is_solid: &F,
) -> (Vec3, bool)
where
    F: Fn(BlockPos) -> bool,
{
    if amount == 0.0 {
        return (position, false);
    }

    let mut target = position;

    match axis {
        Axis::X => target.x += amount,
        Axis::Y => target.y += amount,
        Axis::Z => target.z += amount,
    }

    if !check_collision_at(target, half_extents, is_solid) {
        return (target, false);
    }

    let mut low = 0.0_f32;
    let mut high = 1.0_f32;

    for _ in 0..10 {
        let mid = (low + high) * 0.5;

        let mut test = position;

        match axis {
            Axis::X => test.x += amount * mid,
            Axis::Y => test.y += amount * mid,
            Axis::Z => test.z += amount * mid,
        }

        if check_collision_at(test, half_extents, is_solid) {
            high = mid;
        } else {
            low = mid;
        }
    }

    let mut result = position;

    match axis {
        Axis::X => result.x += amount * low,
        Axis::Y => result.y += amount * low,
        Axis::Z => result.z += amount * low,
    }

    (result, true)
}

/// 单帧最多拆分的子步数，防止极端速度下拆分爆炸。
const MAX_SUBSTEPS: usize = 32;

/// 使用指定方块查询执行玩家移动。
fn move_with_collision_query<F>(
    position: Vec3,
    velocity: Vec3,
    dt: f32,
    half_extents: Vec3,
    is_solid: F,
) -> (Vec3, bool)
where
    F: Fn(BlockPos) -> bool,
{
    debug_assert!(dt >= 0.0, "time step must be non-negative: {dt}");
    debug_assert!(
        half_extents.min_element() > 0.0,
        "half extents must be positive: {half_extents:?}"
    );

    if dt == 0.0 {
        return (position, false);
    }

    // 单轴位移一旦超过碰撞箱最小边长，就可能整块越过方块而不发生重叠。
    // 按最小边长的一半拆成子步，保证相邻两步的 AABB 必然相接。
    let safe_step = half_extents.min_element();
    let max_axis_move = velocity.abs().max_element() * dt;
    let substeps = ((max_axis_move / safe_step).ceil() as usize).clamp(1, MAX_SUBSTEPS);
    let sub_dt = dt / substeps as f32;

    let mut position = position;
    let mut grounded = false;

    for _ in 0..substeps {
        let (next, _) = move_axis(
            position,
            velocity.x * sub_dt,
            half_extents,
            Axis::X,
            &is_solid,
        );
        position = next;

        let (next, _) = move_axis(
            position,
            velocity.z * sub_dt,
            half_extents,
            Axis::Z,
            &is_solid,
        );
        position = next;

        let (next, blocked_y) = move_axis(
            position,
            velocity.y * sub_dt,
            half_extents,
            Axis::Y,
            &is_solid,
        );
        position = next;

        if velocity.y < 0.0 && blocked_y {
            grounded = true;
        }
    }

    (position, grounded)
}

/// 世界实际移动入口。
pub fn move_with_collision(
    position: Vec3,
    velocity: Vec3,
    dt: f32,
    world: &WorldData,
) -> (Vec3, bool) {
    move_with_collision_query(position, velocity, dt, player_half_extents(), |pos| {
        world_is_solid(world, pos)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cj_runtime::WorldSeed;

    fn solid_block(target: BlockPos) -> impl Fn(BlockPos) -> bool {
        move |pos| pos == target
    }

    #[test]
    fn controlled_wall_blocks_x_movement() {
        let position = Vec3::new(0.0, 0.5, 0.5);
        let half_extents = Vec3::splat(0.25);

        let (result, grounded) = move_with_collision_query(
            position,
            Vec3::new(2.0, 0.0, 0.0),
            1.0,
            half_extents,
            solid_block(BlockPos::new(1, 0, 0)),
        );

        assert!(
            result.x <= 1.0 - half_extents.x,
            "player should stop flush against the wall: {}",
            result.x
        );
        assert!(!grounded);
    }

    #[test]
    fn move_z_can_continue_when_x_is_blocked() {
        let position = Vec3::new(0.0, 0.5, 0.0);
        let half_extents = Vec3::splat(0.25);

        let (result, grounded) = move_with_collision_query(
            position,
            Vec3::new(2.0, 0.0, 2.0),
            1.0,
            half_extents,
            |pos| pos.x == 1 && pos.y == 0,
        );

        assert!(
            result.x <= 1.0 - half_extents.x,
            "x movement should stay blocked by the wall plane: {}",
            result.x
        );

        assert!(
            result.z > 1.0,
            "z movement should continue independently: {}",
            result.z
        );

        assert!(!grounded);
    }

    #[test]
    fn downward_collision_reports_grounded() {
        let position = Vec3::new(0.5, 1.5, 0.5);
        let half_extents = Vec3::splat(0.25);

        let (result, grounded) = move_with_collision_query(
            position,
            Vec3::new(0.0, -2.0, 0.0),
            1.0,
            half_extents,
            |pos| pos.y == 0 && pos.x == 0 && pos.z == 0,
        );

        assert!(grounded);

        assert!(
            (result.y - 1.25).abs() < 0.01,
            "player should stop on top of block: {}",
            result.y
        );
    }

    #[test]
    fn upward_collision_does_not_report_grounded() {
        let position = Vec3::new(0.5, 1.5, 0.5);
        let half_extents = Vec3::splat(0.25);

        let (result, grounded) = move_with_collision_query(
            position,
            Vec3::new(0.0, 2.0, 0.0),
            1.0,
            half_extents,
            |pos| pos == BlockPos::new(0, 3, 0),
        );

        assert!(!grounded);
        assert!(
            result.y <= 3.0 - half_extents.y,
            "player should stop flush below ceiling block: {}",
            result.y
        );
    }

    #[test]
    fn negative_coordinates_are_handled() {
        let half_extents = Vec3::splat(0.25);

        let wall = BlockPos::new(-1, 0, 0);

        // 玩家从 x=0 向负方向移动，经过 x=-1 所在的方块。
        let position = Vec3::new(0.5, 0.5, 0.5);

        let (result, _) = move_with_collision_query(
            position,
            Vec3::new(-2.0, 0.0, 0.0),
            1.0,
            half_extents,
            |pos| pos == wall,
        );

        assert!(
            result.x > 0.0,
            "player should stop on the positive side of the negative-x wall: {}",
            result.x
        );
    }

    #[test]
    fn unloaded_chunk_is_not_solid() {
        let world = WorldData::new(WorldSeed::from_seed(0));

        let pos = BlockPos::new(10_000, 128, 10_000);

        assert!(!world_is_solid(&world, pos));
    }

    #[test]
    fn outside_vertical_world_bounds_is_solid() {
        let world = WorldData::new(WorldSeed::from_seed(0));

        let below = BlockPos::new(0, WORLD_BOTTOM - 1, 0);
        let above = BlockPos::new(0, WORLD_TOP, 0);

        assert!(world_is_solid(&world, below));
        assert!(world_is_solid(&world, above));
    }
}
