use avian3d::{collision::collider::*, dynamics::rigid_body::*};
use bevy::{math::FloatPow, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::{GameLayer, player::Player};

pub fn fox_plugin(app: &mut App) {
    app.insert_resource(FoxResource {
        max_attack_distance: 2.,
        run_speed: 3.,
        attack_time: 1.,
        cooldown_time: 1.,
    })
    .add_systems(Update, (chasing_player, attacking_player, cooling_down));
}

#[derive(AssetCollection, Resource)]
pub struct FoxAssets {
    #[asset(path = "Fox.glb#Scene0")]
    fox_scene: Handle<WorldAsset>,
}

#[derive(Resource)]
struct FoxResource {
    run_speed: f32,
    max_attack_distance: f32,
    attack_time: f32,
    cooldown_time: f32,
}

#[derive(Component, Default, Clone, Reflect)]
#[require(CollisionLayers::new(GameLayer::Enemy, LayerMask::ALL))]
pub struct Fox;

pub mod hostile_state {
    use bevy::{ecs::component::Component, reflect::Reflect};

    #[derive(Component, Default, Clone, Reflect)]
    pub struct Chasing;

    #[derive(Component, Default, Clone, Reflect)]
    pub struct Attacking {
        pub time: f32,
    }

    #[derive(Component, Default, Clone, Reflect)]
    pub struct Cooldown {
        pub time: f32,
    }
}

pub fn spawn_fox(mut command: Commands, fox_assets: Res<FoxAssets>) {
    command.spawn_scene(bsn! {
            #Fox
            Fox
            Visibility
            Children [
                WorldAssetRoot({fox_assets.fox_scene.clone()})
                Transform::from_scale(Vec3::splat(0.05))
                // the fox model is backwards
                .with_rotation(Quat::from_rotation_y(Rot2::degrees(180.).as_radians()))
            ]
            RigidBody::Dynamic
            Children [
                Collider::cylinder(0.75, 0.5)
                Transform::from_xyz(0., 0.5, 0.) // shift collider by its height
            ]
            LockedAxes::new().lock_rotation_x().lock_rotation_z() // only allow rotation around y axis
            LinearVelocity::ZERO
            Transform::from_xyz(10.0, 100.0, 0.0)
            hostile_state::Chasing
    });
}

fn chasing_player(
    mut commands: Commands,
    hostiles: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            Entity,
            Option<&hostile_state::Cooldown>,
            Option<&hostile_state::Attacking>,
        ),
        (Without<Player>, With<hostile_state::Chasing>),
    >,
    players: Query<&Transform, With<Player>>,
    fox_resource: Res<FoxResource>,
) {
    for (mut fox_transform, mut fox_velocity, fox_entity, fox_cooldown, fox_attacking) in hostiles {
        for player_transform in players {
            let displacement = player_transform.translation - fox_transform.translation;

            fox_velocity.0 = fox_velocity
                .0
                .with_xz((displacement.normalize() * fox_resource.run_speed).xz());

            fox_transform.rotation = fox_transform
                .looking_at(
                    Vec3::new(
                        player_transform.translation.x,
                        fox_transform.translation.y, // avoid pitch/roll by assuming same height
                        player_transform.translation.z,
                    ),
                    fox_transform.up(),
                )
                .rotation;

            if displacement.length_squared() < fox_resource.max_attack_distance.squared()
                && fox_cooldown.is_none()
                && fox_attacking.is_none()
            {
                commands
                    .entity(fox_entity)
                    .insert(hostile_state::Attacking { time: 0. });
                println!("About to attack!");
            }
        }
    }
}

fn attacking_player(
    mut commands: Commands,
    hostiles: Query<(Entity, &mut hostile_state::Attacking), Without<Player>>,
    fox_resource: Res<FoxResource>,
    time: Res<Time>,
) {
    for (fox_entity, mut attacking) in hostiles {
        attacking.time += time.delta_secs();
        if attacking.time > fox_resource.attack_time {
            println!("Chomp!");
            commands
                .entity(fox_entity)
                .remove::<hostile_state::Attacking>()
                .insert(hostile_state::Cooldown { time: 0. });
        }
    }
}

fn cooling_down(
    mut commands: Commands,
    hostiles: Query<(Entity, &mut hostile_state::Cooldown), Without<Player>>,
    fox_resource: Res<FoxResource>,
    time: Res<Time>,
) {
    for (fox_entity, mut cooldown) in hostiles {
        cooldown.time += time.delta_secs();
        if cooldown.time > fox_resource.cooldown_time {
            commands
                .entity(fox_entity)
                .remove::<hostile_state::Cooldown>();
            println!("Cooldown over!");
        }
    }
}
