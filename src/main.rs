#![allow(clippy::type_complexity)]

use avian3d::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, feathers::FeathersPlugins, prelude::*};
use bevy_ahoy::AhoyPlugins;
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_mod_outline::OutlinePlugin;
use bevy_seedling::SeedlingPlugins;

mod backpack;
mod bounding_polyhedron;
mod camera;
mod cauldron;
mod default_scene;
mod fox;
mod interactable;
mod leditor;
mod main_menu;
mod pickups;
mod player;
mod prelude;
mod rand;
mod timer;
mod wood_grain;

#[allow(unused)]
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GlobalStates {
    #[default]
    AssetLoading,
    MainMenu,
    LevelEditor,
    Game,
}

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Default,
    Player,
    Enemy,
    Ground,
    Cauldron,
    Pickups,
    Trigger,
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
            OutlinePlugin::JUMP_FLOOD,
            SeedlingPlugins,
            FeathersPlugins,
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
            wood_grain::wood_grain_plugin,
            timer::timer_plugin,
            leditor::leditor_plugin,
            main_menu::main_menu_plugin,
        ))
        .add_loading_state(
            LoadingState::new(GlobalStates::AssetLoading)
                .continue_to_state(GlobalStates::MainMenu)
                .load_collection::<player::PlayerAssets>()
                .load_collection::<default_scene::Cubemap>()
                .load_collection::<default_scene::SceneAssets>()
                .load_collection::<pickups::berries::BerryAssets>()
                .load_collection::<fox::FoxAssets>()
                .load_collection::<cauldron::CauldronAssets>()
                .load_collection::<main_menu::MainMenuAssets>(),
        )
        .add_systems(OnEnter(GlobalStates::MainMenu), spawn_camera)
        .init_state::<GlobalStates>()
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}
