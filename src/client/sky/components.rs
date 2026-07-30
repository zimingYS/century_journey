//! 定义天空、太阳、月亮和主光源表现实体的标记组件。

use bevy::prelude::*;

/// 太阳标记组件
#[derive(Component)]
pub struct Sun;

/// 月亮标记组件
#[derive(Component)]
pub struct Moon;

/// 太阳3D Mesh标记组件
#[derive(Component)]
pub struct SunMesh;

/// 月亮3D Mesh标记组件
#[derive(Component)]
pub struct MoonMesh;

/// 星空实体标记组件
#[derive(Component)]
pub struct Stars;
