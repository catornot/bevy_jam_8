use avian3d::prelude::*;
use bevy::prelude::*;

use crate::interactable::Interactable;
use crate::pickups::berries::{BerryAssets, berry_scene};
use crate::prelude::*;

pub mod berries;

pub fn pickups_plugin(app: &mut App) {
    app.add_plugins(berries::berries_plugin)
        .add_systems(OnEnter(GlobalStates::Game), spawn_pickups);
}

#[derive(Component, Clone, Copy, Default)]
#[require(
    RigidBody,
    Collider,
    Interactable,
    CollisionLayers::new(GameLayer::Pickups, LayerMask::ALL)
)]
pub struct Pickup;

pub fn spawn_pickups(mut commands: Commands, berry: Res<BerryAssets>) {
    commands.spawn_scene(bsn! {
        Pickup
        @berry_scene(berry.as_ref())
        RigidBody::Dynamic
        Transform::from_xyz(2., 10., 10.)
    });

    commands.spawn_scene(bsn! {
        Pickup
        @berry_scene(berry.as_ref())
        RigidBody::Dynamic
        Transform::from_xyz(2., 10., 10.)
    });
}
