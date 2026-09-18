//! cj-render 插件

use crate::camera::spawn_camera_and_light;
use crate::terrain::spawn_section_meshes;
use bevy::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn_section_meshes, spawn_camera_and_light));
    }
}
