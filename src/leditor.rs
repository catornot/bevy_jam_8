pub use bevy::prelude::*;

use crate::prelude::*;

pub fn leditor_plugin(app: &mut App) {
    app.add_systems(OnEnter(GlobalStates::LevelEditor), spawn_ledtior);
}

pub fn spawn_ledtior() {}
