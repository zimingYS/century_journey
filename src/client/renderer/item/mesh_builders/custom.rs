use bevy::prelude::*;

pub struct CustomItemMeshBuilder;

impl CustomItemMeshBuilder {
    pub fn build_mesh(_path: &str) -> Option<Mesh> {
        None
    }
}
