use crate::sets::GameSet;
use crate::tools::inspect::{spawn_inspect_readout, update_inspect_readout};
use bevy::prelude::*;

pub struct GameToolsPlugin;

impl Plugin for GameToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_inspect_readout)
            .add_systems(Update, update_inspect_readout.in_set(GameSet::Domain));
    }
}
