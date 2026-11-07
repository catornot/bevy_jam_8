use avian3d::{parry::utils::hashset::HashSet, prelude::*};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_mod_outline::{OutlineVolume, PropagateOutline};

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
            set_selected_intractable_s.run_if(in_state(GlobalStates::Game)),
        );
}

#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct InteractableRayCaster;

#[derive(Component, Clone, Copy, Default, Reflect)]
#[require(OutlineVolume { visible: false, width: 0.5, colour: Color::srgb(0.0, 1.0, 0.0) }, PropagateOutline)]
pub struct Interactable;

#[derive(Component, Clone, Copy, Default, Reflect)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
pub struct Selected;

impl Selected {
    fn on_add(mut world: DeferredWorld<'_>, ctx: HookContext) {
        let mut commands = world.commands();
        let Ok(mut ent_cmds) = commands.get_entity(ctx.entity) else {
            return;
        };
        ent_cmds.insert(OutlineVolume {
            visible: true,
            width: 5.,
            colour: Color::srgb(0.0, 1.0, 0.1),
        });
    }

    fn on_remove(mut world: DeferredWorld<'_>, ctx: HookContext) {
        let mut commands = world.commands();
        let Ok(mut ent_cmds) = commands.get_entity(ctx.entity) else {
            return;
        };
        ent_cmds.insert(OutlineVolume {
            visible: false,
            width: 5.,
            colour: Color::srgb(0.0, 1.0, 0.0),
        });
    }
}

#[derive(Event, Clone, Copy, Default, Reflect)]
pub struct Interacted;

#[derive(Debug, InputAction, Reflect)]
#[action_output(bool)]
pub struct UseAction;

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
