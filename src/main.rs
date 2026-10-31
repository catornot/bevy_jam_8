use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_ahoy::AhoyPlugins;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod default_scene;
mod player;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin,
            EnhancedInputPlugin,
            AhoyPlugins::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            player::player_plugin,
            default_scene::default_scene_plugin,
        ))
        .run();
}
