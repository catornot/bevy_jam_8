#![allow(clippy::type_complexity)]

use avian3d::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_ahoy::AhoyPlugins;
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod backpack;
mod bounding_polyhedron;
mod camera;
mod cauldron;
mod default_scene;
mod fox;
mod interactable;
mod pickups;
mod player;
mod prelude;
mod rand;

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
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_plugins((
            player::player_plugin,
            default_scene::default_scene_plugin,
            pickups::pickups_plugin,
            camera::camera_plugin,
            cauldron::cauldron_plugin,
            rand::rand_plugin,
            bounding_polyhedron::bounding_polyhedron_plugin,
            backpack::backpack_plugin,
            fox::fox_plugin,
            interactable::interactable_plugin,
        ))
        .add_loading_state(
            LoadingState::new(GlobalStates::AssetLoading)
                .continue_to_state(GlobalStates::Game)
                .load_collection::<player::PlayerAssets>()
                .load_collection::<default_scene::Cubemap>()
                .load_collection::<pickups::berries::BerryAssets>()
                .load_collection::<fox::FoxAssets>()
                .load_collection::<cauldron::CauldronAssets>(),
        )
        .init_state::<GlobalStates>()
        .run();
}
