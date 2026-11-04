use avian3d::{
    parry::{self, query::RayCast},
    prelude::*,
};
use bevy::{
    ecs::{lifecycle::HookContext, relationship::Relationship, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use rand::RngExt;
use std::{borrow::Cow, f32::consts::PI};

use crate::{bounding_polyhedron::BoundingPolyhedron, interactable::Selected};
use crate::{pickups::Pickup, rand::Rand};

pub fn backpack_plugin(app: &mut App) {
    app.register_type::<Backpack>()
        .register_type::<Backpacked>()
        .register_type::<PickupItem>()
        .add_systems(Update, (move_to_backpack, calculate_backpacking_info))
        .add_observer(deposit_in_backpack);
}

#[derive(Debug, InputAction, Reflect)]
#[action_output(bool)]
pub struct PickupItem;

#[derive(Debug, Component, Default, Clone, Reflect)]
#[relationship_target(relationship = Backpacked)]
pub struct Backpack {
    #[relationship]
    items: Vec<Entity>,
}

/// to which entity should items be parented
/// must still exist within the backpack's space
/// better yet be aligned to the origin of the backpack
#[derive(Debug, Component, Clone, Reflect)]
pub struct BackpackAttachment(Entity);

impl BackpackAttachment {
    pub fn new(attachment: Entity) -> Self {
        Self(attachment)
    }
}

#[derive(Component, Clone, Reflect)]
#[relationship(relationship_target = Backpack)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
#[require(MovingToBackpack)]
pub struct Backpacked(Entity);

impl Backpacked {
    pub fn new(backpack_ent: Entity) -> Self {
        Self(backpack_ent)
    }

    fn on_add(mut world: DeferredWorld<'_>, ctx: HookContext) {
        let mut commands = world.commands();
        let Ok(mut ent_cmds) = commands.get_entity(ctx.entity) else {
            return;
        };
        ent_cmds.insert(RigidBodyDisabled);
        ent_cmds.insert(ColliderDisabled);
    }

    fn on_remove(mut world: DeferredWorld<'_>, ctx: HookContext) {
        let mut commands = world.commands();
        let Ok(mut ent_cmds) = commands.get_entity(ctx.entity) else {
            return;
        };
        ent_cmds.remove::<RigidBodyDisabled>();
        ent_cmds.remove::<ColliderDisabled>();
    }
}

#[derive(Component, Default, Clone)]
pub struct MovingToBackpack;

#[derive(Component, Clone)]
pub struct CalculatedBackpackDistance {
    offset: Vec3,
    starting_distance: f32,
}

fn deposit_in_backpack(
    trigger: On<Fire<PickupItem>>,
    mut commands: Commands,
    for_pickup: Query<Entity, (With<Selected>, With<Pickup>)>,
) {
    if !trigger.value {
        return;
    }

    for ent in for_pickup {
        if let Ok(mut cmds) = commands.get_entity(ent) {
            cmds.insert(Backpacked::new(trigger.context));
        }
    }
}

fn calculate_backpacking_info(
    mut commands: Commands,
    mut rand: ResMut<Rand>,
    pickups: Query<(&Backpacked, Entity, &Transform), (Without<Backpack>, With<MovingToBackpack>)>,
    targets: Query<(&Transform, &BoundingPolyhedron), With<Backpack>>,
) {
    for (ent, bundle) in
        pickups
            .into_iter()
            .filter_map(|(backpacked, pickup, backped_transform)| {
                let Ok((parent_transform, bounding)) = targets.get(backpacked.get()) else {
                    // not really possible but ok
                    return None;
                };

                let dir = Dir3::new(
                    (rand.random::<Vec3>() * Vec3::splat(PI))
                        .clamp(Vec3::splat(0.), Vec3::splat(PI)),
                )
                .unwrap_or(Dir3::Y);

                // this is in the space of the model
                let offset = dir * 20.;

                let result = if let Some(distance) = bounding.get().cast_local_ray(
                    &parry::query::Ray {
                        origin: offset,
                        dir: -dir.as_vec3(),
                    },
                    (dir * 20.).length_squared(),
                    true,
                ) {
                    CalculatedBackpackDistance {
                        offset: -(distance * dir) + offset,
                        starting_distance: backped_transform
                            .translation
                            .distance(parent_transform.translation),
                    }
                } else {
                    let closest_point = bounding
                        .get()
                        .points()
                        .iter()
                        .map(|point| (point, point.distance_squared(offset)))
                        .reduce(|point1, point2| {
                            if point1.1.total_cmp(&point2.1).is_lt() {
                                point1
                            } else {
                                point2
                            }
                        })
                        .map(|(point, _)| point)
                        .unwrap_or(&Vec3::ZERO);

                    CalculatedBackpackDistance {
                        offset: *closest_point,
                        starting_distance: backped_transform
                            .translation
                            .distance(parent_transform.translation),
                    }
                };

                Some((pickup, result))
            })
    {
        if let Ok(mut cmds) = commands.get_entity(ent) {
            cmds.insert(bundle);
        }
    }
}

fn move_to_backpack(
    mut commands: Commands,
    mut gizmos: Gizmos,
    moving: Query<
        (
            &mut Transform,
            &Backpacked,
            &CalculatedBackpackDistance,
            Entity,
        ),
        Without<Backpack>,
    >,
    targets: Query<(&Transform, Entity, Option<&BackpackAttachment>), With<Backpack>>,
    possible_attachment_target: Query<
        (Entity, &GlobalTransform),
        (Without<Backpack>, Without<CalculatedBackpackDistance>),
    >, // not sure if this is expensive
    time: Res<Time>,
) {
    for (mut transform, backpacked, backpacking_info, ent) in moving {
        let Ok((parent_transform, parent_ent, attachment_info)) = targets.get(backpacked.get())
        else {
            continue;
        };

        let parent_transform = attachment_info
            .and_then(|ent| possible_attachment_target.get(ent.0).ok().map(|(_, t)| t))
            .map(|global| Cow::Owned(global.compute_transform()))
            .unwrap_or(Cow::Borrowed(parent_transform));

        let distance = transform.translation.distance(parent_transform.translation);
        let target = parent_transform.transform_point(backpacking_info.offset);
        transform
            .translation
            .x
            .smooth_nudge(&target.x, 10f32.ln(), time.delta_secs() * 2.);
        transform.translation.y.smooth_nudge(
            &(target.y
                + ((distance / backpacking_info.starting_distance) * PI)
                    .clamp(0., PI)
                    .sin()
                    * 5.),
            10f32.ln(),
            time.delta_secs() * 2.,
        );
        transform
            .translation
            .z
            .smooth_nudge(&target.z, 10f32.ln(), time.delta_secs() * 2.);

        gizmos.line(transform.translation, target, Color::WHITE);

        if (target.xz() - transform.translation.xz()).length() < 0.05
            && let Ok(mut pickup_cmds) = commands.get_entity(ent)
        {
            pickup_cmds.remove::<MovingToBackpack>();
            pickup_cmds.remove::<CalculatedBackpackDistance>();
            transform.translation = backpacking_info.offset;

            if let Some(player_model_ent) = attachment_info
                .and_then(|ent| possible_attachment_target.get(ent.0).ok().map(|(e, _)| e))
            {
                if let Ok(mut parent_cmds) = commands.get_entity(player_model_ent) {
                    parent_cmds.add_child(ent);
                }
            } else if let Ok(mut parent_cmds) = commands.get_entity(parent_ent) {
                parent_cmds.add_child(ent);
            } else {
                bevy::log::error!("failed to assign parent to pickup : {parent_ent:?} and {ent:?}");
            }
        }
    }
}
