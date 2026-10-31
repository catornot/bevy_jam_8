use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_ahoy::AhoyPlugins;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod default_scene;
mod player;

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Hedgehog;

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
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let hedgehog = asset_server.load(GltfAssetLabel::Scene(0).from_asset("hedgehog.glb"));
    commands.spawn((
        WorldAssetRoot(hedgehog),
        Hedgehog,
        Transform::from_translation(Vec3::new(0., 2., 0.)),
        RigidBody::Dynamic,
        Collider::cuboid(2., 2., 2.),
    ));
}

fn update(mut query: Query<&mut Transform, With<Hedgehog>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
        transform.rotate_x(time.delta_secs() * 20.);
    }
}
