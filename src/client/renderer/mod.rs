use bevy::prelude::*;

pub mod held_renderer;
pub mod mesh_cache;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<mesh_cache::HeldMeshCache>();
    }
}
