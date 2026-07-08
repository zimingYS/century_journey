use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HeldRenderDefinition {
    #[serde(rename = "empty")]
    #[default]
    Empty,
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "flat_item")]
    FlatItem {
        #[serde(default = "default_thickness")]
        thickness: f32,
    },
    #[serde(rename = "model")]
    Model { path: String },
}

fn default_thickness() -> f32 {
    0.05
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeldItemConfig {
    #[serde(default)]
    pub render: HeldRenderDefinition,
    #[serde(default = "default_fp_translation")]
    pub first_person_translation: [f32; 3],
    #[serde(default)]
    pub first_person_rotation: [f32; 3],
    #[serde(default = "default_fp_scale")]
    pub first_person_scale: f32,
    #[serde(default)]
    pub animations: AnimationConfig,
}

fn default_fp_translation() -> [f32; 3] {
    [0.02, -0.02, -0.04]
}

fn default_fp_scale() -> f32 {
    0.65
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationConfig {
    #[serde(default)]
    pub swing: bool,
    #[serde(default)]
    pub eat: bool,
    #[serde(default)]
    pub use_anim: bool,
    #[serde(default)]
    pub spyglass: bool,
}

impl Default for HeldItemConfig {
    fn default() -> Self {
        Self {
            render: HeldRenderDefinition::Empty,
            first_person_translation: default_fp_translation(),
            first_person_rotation: [0.0; 3],
            first_person_scale: default_fp_scale(),
            animations: AnimationConfig::default(),
        }
    }
}

impl HeldItemConfig {
    pub fn to_transform(&self) -> Transform {
        Transform {
            translation: Vec3::from(self.first_person_translation),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                self.first_person_rotation[0].to_radians(),
                self.first_person_rotation[1].to_radians(),
                self.first_person_rotation[2].to_radians(),
            ),
            scale: Vec3::splat(self.first_person_scale),
        }
    }

    pub fn default_block() -> Self {
        Self {
            render: HeldRenderDefinition::Block,
            first_person_translation: [0.0, -0.04, -0.7],
            first_person_rotation: [0.0, 15.0, 0.0],
            first_person_scale: 0.4,
            animations: AnimationConfig {
                swing: true,
                ..default()
            },
        }
    }

    pub fn default_tool(thickness: f32) -> Self {
        Self {
            render: HeldRenderDefinition::FlatItem { thickness },
            first_person_translation: [0.0, 0.3, -0.3],
            first_person_rotation: [0.0, -60.0, 30.0],
            first_person_scale: 1.0,
            animations: AnimationConfig {
                swing: true,
                ..default()
            },
        }
    }

    pub fn default_flat(thickness: f32) -> Self {
        Self {
            render: HeldRenderDefinition::FlatItem { thickness },
            first_person_translation: [0.0, 0.3, -0.3],
            first_person_rotation: [0.0, -60.0, 30.0],
            first_person_scale: 1.0,
            animations: AnimationConfig::default(),
        }
    }

    pub fn default_model(path: &str) -> Self {
        Self {
            render: HeldRenderDefinition::Model {
                path: path.to_string(),
            },
            first_person_translation: [0.0, 0.3, -0.3],
            first_person_rotation: [0.0, -60.0, 30.0],
            first_person_scale: 1.0,
            animations: AnimationConfig::default(),
        }
    }
}
