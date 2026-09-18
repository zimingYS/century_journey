//! CenturyJourney —— 程序入口

use bevy::prelude::*;

mod bootstrap;
mod plugin;
mod settings;

fn main() {
    App::new().add_plugins(plugin::AppPlugin).run();
}
