
use bevy::{math::FloatPow, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::{GlobalStates, player::Player};

pub fn fox_plugin(app: &mut App) {
    app.register_type::<Fox>()
        .insert_resource(FoxResource {
            max_attack_distance: 1.,
            speed: 1.,
        })
        .add_systems(OnEnter(GlobalStates::Game), spawn_fox)
        .add_systems(Update, hunt_player);
}

#[derive(AssetCollection, Resource)]
pub struct FoxAssets {
    #[asset(path = "Fox.glb#Scene0")]
    fox_scene: Handle<WorldAsset>,
}

#[derive(Resource)]
pub struct FoxResource {
    max_attack_distance: f32,
    speed: f32,
}

#[derive(Component, Default, Clone, Reflect)]
pub struct Fox;

pub fn spawn_fox(mut command: Commands, fox_assets: Res<FoxAssets>) {
    command.spawn_scene(bsn! {
            #Fox
            Fox
            WorldAssetRoot({fox_assets.fox_scene.clone()})
            Transform::from_xyz(0.0, 2.0, 0.0).with_scale(Vec3::splat(0.05))
    });
}

pub fn hunt_player(
    foxes: Query<&mut Transform, With<Fox>>,
    players: Query<&Transform, (With<Player>, Without<Fox>)>,
    fox_resource: Res<FoxResource>,
    time: Res<Time>,
) {
    for mut fox_transform in foxes {
        for player_transform in players {
            let displacement = player_transform.translation - fox_transform.translation;
            fox_transform.translation +=
                displacement.normalize() * fox_resource.speed * time.delta_secs();

            if displacement.length_squared() < fox_resource.max_attack_distance.squared() {
                print!("Dead!")
            }
        }
    }
}
