use bevy::prelude::*;
use std::collections::HashMap;

/// Client-only render ownership. Gameplay never reads or writes this resource.
#[derive(Resource, Debug, Default)]
pub struct ClientPresentation {
    pub chunk_entities: HashMap<IVec3, Entity>,
    pub mesh_entities: Vec<Entity>,
    pub material_entities: Vec<Entity>,
    pub render_entities: Vec<Entity>,
}
