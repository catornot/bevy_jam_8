use avian3d::{parry::utils::hashset::HashSet, prelude::*};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

const MAX_INTERACT_DIST: f32 = 5.;

pub fn interactable_plugin(app: &mut App) {
    app.register_type::<Interactable>()
        .register_type::<Selected>()
        .register_type::<InteractableRayCaster>()
        .register_type::<UseAction>()
        .register_type::<Interacted>()
        .add_systems(
            Update,
            (set_selected_intractable_s, highlight_selected).run_if(in_state(GlobalStates::Game)),
        );
}

#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct InteractableRayCaster;

#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct Interactable;

#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct Selected;

#[derive(Event, Clone, Copy, Default, Reflect)]
pub struct Interacted;

#[derive(Debug, InputAction, Reflect)]
#[action_output(bool)]
pub struct UseAction;

pub fn highlight_selected(pickups: Query<&Transform, With<Selected>>, mut gizmos: Gizmos) {
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

pub fn set_selected_intractable_s(
    mut commands: Commands,
    players: Query<(&RayCaster, &RayHits), With<InteractableRayCaster>>,
    pickups_highlighting: Query<Entity, (With<Interactable>, Without<Selected>)>, // stuff to highlight
    pickups_non_highlighting: Query<Entity, (With<Interactable>, With<Selected>)>, // highlighted stuff
    mut highlighted_pickups: Local<HashSet<Entity>>,
) {
    highlighted_pickups.clear();

    for target in players.into_iter().flat_map(|(_, hits)| {
        hits.iter()
            .filter(|hit| hit.distance < MAX_INTERACT_DIST)
            .filter_map(|hit| {
                pickups_highlighting
                    .get(hit.entity)
                    .or_else(|_| pickups_non_highlighting.get(hit.entity))
                    .ok()
            })
    }) {
        highlighted_pickups.insert(target);
        if pickups_highlighting.get(target).is_ok()
            && let Ok(mut ent) = commands.get_entity(target)
        {
            ent.insert(Selected);
        }
    }

    for ent in pickups_non_highlighting
        .into_iter()
        .filter(|ent| !highlighted_pickups.contains(ent))
    {
        if let Ok(mut commands) = commands.get_entity(ent) {
            commands.remove::<Selected>();
        }
    }
}
