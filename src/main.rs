#![allow(clippy::type_complexity)]

use avian3d::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_ahoy::AhoyPlugins;
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod camera;
mod default_scene;
mod pickups;
mod player;
mod prelude;

#[allow(unused)]
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GlobalStates {
    #[default]
    AssetLoading,
    MainMenu,
    Game,
}

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
            pickups::pickups_plugin,
            camera::camera_plugin,
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_loading_state(
            LoadingState::new(GlobalStates::AssetLoading)
                .continue_to_state(GlobalStates::Game)
                .load_collection::<player::PlayerAssets>()
                .load_collection::<pickups::berries::BerryAssets>(),
        )
        .init_state::<GlobalStates>()
        .run();
}
