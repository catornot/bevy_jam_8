use avian3d::{parry::utils::hashset::HashSet, prelude::*};
use bevy::prelude::*;

use crate::pickups::berries::{BerryAssets, berry_scene};
use crate::prelude::*;

pub mod berries;

const MAX_HIGHLIGHT_DIST: f32 = 100.;

#[derive(Component, Clone, Copy, Default)]
#[require(RigidBody, Collider)]
pub struct Pickup;

#[derive(Component, Clone, Copy, Default)]
pub struct PickupRayCaster;

// should probably turn this into a relationship so a pickup ready component is needed
#[derive(Component)]
pub struct HighlightPickup;

pub fn pickups_plugin(app: &mut App) {
    app.add_plugins(berries::berries_plugin)
        .add_systems(OnEnter(GlobalStates::Game), spawn_pickups)
        .add_systems(
            Update,
            (set_highlight_pickups, highlight_pickups).run_if(in_state(GlobalStates::Game)),
        );
}

pub fn highlight_pickups(pickups: Query<&Transform, With<HighlightPickup>>, mut gizmos: Gizmos) {
    for transform in pickups {
        gizmos
            .rounded_cuboid(
                Isometry3d::from_translation(transform.translation),
                Vec3::splat(2.),
                bevy::color::palettes::css::GREEN,
            )
            .edge_radius(0.25)
            .arc_resolution(10);
    }
}

pub fn spawn_pickups(mut commands: Commands, berry: Res<BerryAssets>) {
    commands.spawn_scene(bsn! {
        Pickup
        @berry_scene(berry.as_ref())
        RigidBody::Dynamic
        Transform::from_xyz(2., 10., 10.)
    });
}

pub fn set_highlight_pickups(
    mut command: Commands,
    players: Query<(&RayCaster, &RayHits), With<PickupRayCaster>>,
    pickups_highlighting: Query<Entity, (With<Pickup>, Without<HighlightPickup>)>, // stuff to highlight
    pickups_non_highlighting: Query<Entity, (With<Pickup>, With<HighlightPickup>)>, // highlighted stuff
    mut highlighted_pickups: Local<HashSet<Entity>>,
) {
    highlighted_pickups.clear();

    for target in players.into_iter().flat_map(|(_, hits)| {
        hits.iter()
            .filter(|hit| hit.distance < MAX_HIGHLIGHT_DIST)
            .filter_map(|hit| {
                pickups_highlighting
                    .get(hit.entity)
                    .or_else(|_| pickups_non_highlighting.get(hit.entity))
                    .ok()
            })
    }) {
        highlighted_pickups.insert(target);
        if pickups_highlighting.get(target).is_ok()
            && let Ok(mut ent) = command.get_entity(target)
        {
            ent.insert(HighlightPickup);
        }
    }

    for ent in pickups_non_highlighting
        .into_iter()
        .filter(|ent| !highlighted_pickups.contains(ent))
    {
        if let Ok(mut commands) = command.get_entity(ent) {
            commands.remove::<HighlightPickup>();
        }
    }
}
