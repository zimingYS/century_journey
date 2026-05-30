use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Reflect)]
#[repr(u16)]
pub enum VoxelType{
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Wood = 4,
    Leaves = 5,
    Sand = 6,
    Water = 7,
}

impl VoxelType{
    pub fn from_u16(value: u16) -> Self {
        match value {
            1 => VoxelType::Grass,
            2 => VoxelType::Dirt,
            3 => VoxelType::Stone,
            4 => VoxelType::Wood,
            5 => VoxelType::Leaves,
            6 => VoxelType::Sand,
            7 => VoxelType::Water,
            _ => VoxelType::Air,
        }
    }
    pub(crate) fn get_voxel_color (&self) -> Color {
        match self {
            VoxelType::Grass => Color::srgb(0.2, 0.8, 0.2),
            VoxelType::Dirt => Color::srgb(0.6, 0.4, 0.2),
            VoxelType::Stone => Color::srgb(0.5, 0.5, 0.5),
            VoxelType::Wood => Color::srgb(0.4, 0.3, 0.2),
            VoxelType::Leaves => Color::srgb(0.1, 0.6, 0.1),
            VoxelType::Sand => Color::srgb(0.9, 0.9, 0.6),
            VoxelType::Water => Color::srgba(0.2, 0.4, 0.9, 0.6),
            VoxelType::Air => Color::NONE,
        }
    }
}