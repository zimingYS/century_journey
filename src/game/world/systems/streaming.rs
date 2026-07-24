use crate::content::constant::world::{CHUNK_SIZE, SEA_LEVEL};
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct WorldStreamingConfig {
    pub data_horizontal_radius: i32,
    pub mesh_horizontal_radius: i32,
    pub data_vertical_radius_above: i32,
    pub data_vertical_radius_below: i32,
    pub mesh_vertical_radius_above: i32,
    pub mesh_vertical_radius_below: i32,
    pub surface_priority_radius: i32,
}

impl Default for WorldStreamingConfig {
    fn default() -> Self {
        Self::new(8, 8, 2, 3)
    }
}

impl WorldStreamingConfig {
    /// 根据纯数值配置构造世界流式窗口，避免 Game 依赖 App 配置类型。
    pub fn new(
        render_distance: u32,
        mesh_distance: u32,
        data_vertical_radius_above: u32,
        data_vertical_radius_below: u32,
    ) -> Self {
        let render_distance = as_i32(render_distance.max(1));
        let mesh_horizontal_radius = as_i32(mesh_distance.max(1)).min(render_distance);
        let data_horizontal_radius = render_distance.saturating_add(1);
        let data_vertical_radius_above = as_i32(data_vertical_radius_above);
        let data_vertical_radius_below = as_i32(data_vertical_radius_below);
        let mesh_vertical_radius_above = data_vertical_radius_above.saturating_sub(1);
        let mesh_vertical_radius_below = data_vertical_radius_below.saturating_sub(1);

        Self {
            data_horizontal_radius,
            mesh_horizontal_radius,
            data_vertical_radius_above,
            data_vertical_radius_below,
            mesh_vertical_radius_above,
            mesh_vertical_radius_below,
            surface_priority_radius: 1,
        }
    }

    pub fn chunk_from_world(position: Vec3) -> IVec3 {
        let size = CHUNK_SIZE as f32;
        IVec3::new(
            (position.x / size).floor() as i32,
            (position.y / size).floor() as i32,
            (position.z / size).floor() as i32,
        )
    }

    pub fn rebuild_expected_chunks(
        &self,
        player_chunk_pos: IVec3,
        view_forward_xz: Vec2,
    ) -> (Vec<IVec3>, HashSet<IVec3>) {
        let mut chunks = Vec::with_capacity(self.data_chunk_capacity_estimate());
        let radius = self.data_horizontal_radius;
        let radius_sq = horizontal_radius_sq(radius);

        for x in -radius..=radius {
            for z in -radius..=radius {
                if horizontal_distance_sq(x, z) > radius_sq {
                    continue;
                }
                for y in -self.data_vertical_radius_below..=self.data_vertical_radius_above {
                    chunks.push(player_chunk_pos + IVec3::new(x, y, z));
                }
            }
        }

        self.sort_chunks_by_priority(&mut chunks, player_chunk_pos, view_forward_xz);
        let expected = chunks.iter().copied().collect();
        (chunks, expected)
    }

    pub fn should_mesh_chunk(&self, player_chunk_pos: IVec3, chunk_pos: IVec3) -> bool {
        let delta = chunk_pos - player_chunk_pos;
        horizontal_distance_sq(delta.x, delta.z)
            <= horizontal_radius_sq(self.mesh_horizontal_radius)
            && delta.y >= -self.mesh_vertical_radius_below
            && delta.y <= self.mesh_vertical_radius_above
    }

    fn sort_chunks_by_priority(
        &self,
        chunks: &mut [IVec3],
        player_chunk_pos: IVec3,
        view_forward_xz: Vec2,
    ) {
        chunks.sort_by_key(|&chunk_pos| {
            self.priority_key(chunk_pos, player_chunk_pos, view_forward_xz)
        });
    }

    fn priority_key(
        &self,
        chunk_pos: IVec3,
        player_chunk_pos: IVec3,
        view_forward_xz: Vec2,
    ) -> i64 {
        let delta = chunk_pos - player_chunk_pos;
        let horizontal_sq = horizontal_distance_sq(delta.x, delta.z);
        let vertical = i64::from(delta.y.abs());
        let surface_delta = i64::from((chunk_pos.y - surface_chunk_y()).abs());
        let surface_penalty = if surface_delta <= i64::from(self.surface_priority_radius) {
            0
        } else {
            surface_delta * 8
        };
        let forward_bonus = forward_priority_bonus(delta, view_forward_xz);

        horizontal_sq * 100 + vertical * 36 + surface_penalty - forward_bonus
    }

    fn data_chunk_capacity_estimate(&self) -> usize {
        let diameter = i64::from(self.data_horizontal_radius) * 2 + 1;
        let vertical =
            i64::from(self.data_vertical_radius_above + self.data_vertical_radius_below + 1);
        (diameter * diameter * vertical).max(0) as usize
    }
}

fn as_i32(value: u32) -> i32 {
    value.min(i32::MAX as u32) as i32
}

fn horizontal_radius_sq(radius: i32) -> i64 {
    let radius = i64::from(radius);
    radius * radius
}

fn horizontal_distance_sq(x: i32, z: i32) -> i64 {
    let x = i64::from(x);
    let z = i64::from(z);
    x * x + z * z
}

fn surface_chunk_y() -> i32 {
    SEA_LEVEL.div_euclid(CHUNK_SIZE as i32)
}

fn forward_priority_bonus(delta: IVec3, view_forward_xz: Vec2) -> i64 {
    let to_chunk = Vec2::new(delta.x as f32, delta.z as f32).normalize_or_zero();
    let forward = view_forward_xz.normalize_or_zero();
    if to_chunk == Vec2::ZERO || forward == Vec2::ZERO {
        return 0;
    }

    (to_chunk.dot(forward).max(0.0) * 120.0) as i64
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/systems/streaming.rs"]
mod tests;
