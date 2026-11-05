use avian3d::{collision::collider::Collider, dynamics::rigid_body::*};
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::{GlobalStates, player::Player};

pub fn fox_plugin(app: &mut App) {
    app.register_type::<Fox>()
        .insert_resource(FoxResource {
            max_attack_distance: 1.,
            speed: 3.,
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
            Children [
                WorldAssetRoot({fox_assets.fox_scene.clone()})
                Transform::from_scale(Vec3::splat(0.05)).with_rotation(Quat::from_rotation_y(Rot2::degrees(180.).as_radians()))
            ]
            RigidBody::Dynamic
            Children [
                Collider::cylinder(0.75, 0.5)
                Transform::from_xyz(0., 0.5, 0.)
            ]
            LockedAxes::new().lock_rotation_x().lock_rotation_z()
            LinearVelocity::ZERO
            Transform::from_xyz(10.0, 100.0, 0.0)
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

            fox_transform.rotation = fox_transform
                .looking_at(
                    Vec3::new(
                        player_transform.translation.x,
                        fox_transform.translation.y, // avoid pitch/roll by assuming same height
                        player_transform.translation.z,
                    ),
                    fox_transform.up(),
                )
                .rotation
        }
    }
}
