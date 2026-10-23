//! cj-render 插件

use crate::lighting::spawn_light;
use crate::resources::{DirtySections, SectionMeshes};
use crate::terrain::{rebuild_dirty_sections, spawn_section_meshes};
use bevy::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirtySections>()
            .init_resource::<SectionMeshes>()
            .add_systems(PostStartup, (spawn_section_meshes, spawn_light))
            .add_systems(Update, rebuild_dirty_sections);
    }
}
