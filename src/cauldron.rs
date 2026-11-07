use avian3d::prelude::*;
use bevy::{
    ecs::relationship::Relationship, prelude::*, reflect::TypePath,
    render::render_resource::AsBindGroup, shader::ShaderRef,
};
use bevy_enhanced_input::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::{
    backpack::{Backpack, BackpackAttachment, BackpackConstraints, Backpacked, MovingToBackpack},
    bounding_polyhedron::{BoundingPolyhedronMarker, BoundingPolyhedronSkipMarker},
    interactable::{Interactable, Selected, UseAction},
    pickups::Pickup,
    prelude::*,
    timer::{TimerTimeout, TriggerTimer},
};

const SHADER_ASSET_PATH: &str = "shaders/potion_shader.wesl";

pub fn cauldron_plugin(app: &mut App) {
    app.register_type::<Cauldron>()
        .register_type::<CauldronDrops>()
        .register_type::<CauldronTrigger>()
        .register_type::<CauldronEffect>()
        .add_plugins(MaterialPlugin::<PotionMaterial>::default())
        .add_systems(OnEnter(GlobalStates::Game), spawn_cauldron)
        .add_systems(Update, (sync_material, drop_items, consume_items))
        .add_observer(deposit_in_cauldron)
        .add_observer(on_drop_timeout);
}

#[derive(Component, Clone, FromTemplate, Reflect)]
#[require(CollisionLayers::new(GameLayer::Cauldron, LayerMask::ALL))]
pub struct Cauldron(Laba);

#[derive(Component, Clone, FromTemplate, Reflect)]
pub struct CauldronDrops(Vec3);

#[derive(Component, Clone, FromTemplate, Reflect)]
#[require(CollisionLayers::new(GameLayer::Trigger, GameLayer::Pickups))]
pub struct CauldronTrigger;

#[derive(AssetCollection, Resource)]
pub struct CauldronAssets {
    #[asset(path = "cauldron.glb#Scene0")]
    cauldron: Handle<WorldAsset>,
}

/// the offset in the color space that a certain item produces
#[derive(Debug, Component, Clone, FromTemplate, Reflect)]
pub struct CauldronEffect(pub Vec3);

fn spawn_cauldron(
    mut commands: Commands,
    cauldron_assets: Res<CauldronAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PotionMaterial>>,
) {
    let material = PotionMaterial {
        color: Vec4::new(0.86644, -0.233887, 0.179498, 0.),
        positions: [Vec2::new(-1., -1.).extend(-1.); 8],
    };
    let color: Hsva = Color::srgba_u8(0, 255, 0, 255).into();
    commands.spawn_scene(bsn! {
        #Cauldron
        Cauldron(color)
        WorldAssetRoot({cauldron_assets.cauldron.clone()})
        Transform::from_xyz(0., 1.5, 10.)
        BoundingPolyhedronMarker::new(0.9)
        RigidBody::Static
        Collider::cylinder(1., 1.)
        Interactable
        Children [
            #LiquidPlane
            Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)).with_translation(Vec3::new(0., 0.25, 0.))
            BoundingPolyhedronSkipMarker
            Mesh3d({meshes.add(Circle::new(0.9))})
            MeshMaterial3d<PotionMaterial>({materials.add(material)})
            --
            #Trigger
            CauldronTrigger
            Sensor
            RigidBody::Static
            Collider::cylinder(0.8, 0.5)
            CollisionEventsEnabled
        ]
        Backpack
        BackpackConstraints {
            static_dir: {Some(Dir3::Y)}
        }
    });
}

fn sync_material(
    cauldrons: Query<(&Cauldron, &Children)>,
    potion_planes: Query<&MeshMaterial3d<PotionMaterial>>,
    drops: Query<&CauldronDrops>,
    mut materials: ResMut<Assets<PotionMaterial>>,
) {
    for (cauldron, children) in cauldrons {
        let Some(material_handle) = children
            .iter()
            .find_map(|child| potion_planes.get(child).ok())
        else {
            continue;
        };

        if let Some(mut material) = materials.get_mut(material_handle.id()) {
            let color: Oklaba = cauldron.0.into();
            material
                .color
                .smooth_nudge(&color.to_vec4(), 10f32.ln(), 0.01);

            let mut max = 0;
            for (i, &CauldronDrops(offset)) in children
                .into_iter()
                .copied()
                .filter_map(|child| drops.get(child).ok())
                .take(8)
                .enumerate()
            {
                if let Some(slot) = material.positions.get_mut(i) {
                    *slot = (offset + Vec2::splat(0.5).extend(0.)) / Vec2::splat(0.9).extend(5.);
                }
                max = i + 1;
            }

            for index in max.min(8)..8 {
                material.positions[index] = Vec3::splat(-1.);
            }
        }
    }
}

fn deposit_in_cauldron(
    trigger: On<Fire<UseAction>>,
    mut commands: Commands,
    cauldrons: Query<Entity, (With<Cauldron>, With<Selected>)>,
    parents: Query<&GlobalTransform, Or<(With<Backpack>, With<BackpackAttachment>)>>,
    mut picked_up: Query<
        (&mut Transform, Entity, &Backpacked),
        (With<Pickup>, Without<MovingToBackpack>),
    >,
) {
    if !trigger.value {
        return;
    }
    let Some(cauldron) = cauldrons.into_iter().next() else {
        return;
    };

    for (mut transform, ent, backpacked) in &mut picked_up {
        if backpacked.get() == trigger.context {
            commands.entity(cauldron).add_one_related::<Backpacked>(ent);
            commands.entity(ent).insert(MovingToBackpack);
            commands.entity(ent).remove::<ChildOf>();

            if let Ok(parent_transform) = parents.get(backpacked.get()) {
                transform.translation = parent_transform.transform_point(transform.translation);
            }
        }
    }
}

fn drop_items(
    mut commands: Commands,
    attached: Query<(&Backpacked, Entity), Without<MovingToBackpack>>,
    cauldrons: Query<(), With<Cauldron>>,
) {
    for (parent, ent) in attached {
        if cauldrons.get(parent.get()).is_err() {
            continue;
        }

        if let Ok(mut cauldron_cmds) = commands.get_entity(ent) {
            cauldron_cmds.remove_related::<Backpacked>(&[ent]);
            cauldron_cmds.detach_child(ent);
        }

        if let Ok(mut pickups_cmds) = commands.get_entity(ent) {
            let mask =
                (LayerMask::ALL & !GameLayer::Pickups.to_bits()) & !GameLayer::Cauldron.to_bits();
            pickups_cmds.insert(CollisionLayers::new(GameLayer::Pickups, mask));
            pickups_cmds.remove::<Backpacked>();
        }
    }
}

fn consume_items(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    pickups: Query<(&GlobalTransform, &CauldronEffect), (With<Pickup>, Without<Backpacked>)>,
    sensors: Query<&ChildOf, With<Sensor>>,
    mut cauldrons: Query<(&mut Cauldron, &Transform), Without<CauldronEffect>>,
) {
    for collision_event in collision_reader.read() {
        let Ok(sensor) = sensors.get(collision_event.collider2) else {
            continue;
        };

        let Ok(pickup) = pickups.get(collision_event.collider1) else {
            continue;
        };

        let Ok((mut cauldron, cauldron_transform)) = cauldrons.get_mut(sensor.parent()) else {
            continue;
        };

        commands.entity(collision_event.collider1).despawn();

        let timer = commands
            .spawn((
                TriggerTimer::new(2.),
                CauldronDrops(
                    (cauldron_transform.translation - pickup.0.translation())
                        .xz()
                        .extend(1.),
                ),
            ))
            .id();
        commands.entity(sensor.parent()).add_child(timer);

        cauldron.0.lightness =
            (cauldron.0.lightness.algebraic_sub(0.3) + pickup.1.0.x).rem_euclid(0.7) + 0.3;
        cauldron.0.a = (cauldron.0.a.algebraic_sub(0.2) + pickup.1.0.y).rem_euclid(0.8) + 0.2;
        cauldron.0.b = (cauldron.0.b.algebraic_sub(0.2) + pickup.1.0.z).rem_euclid(0.8) + 0.2;
    }
}

fn on_drop_timeout(event: On<TimerTimeout>, mut commands: Commands) {
    commands.entity(event.event().0).despawn();
}

#[derive(Debug, Asset, Default, TypePath, AsBindGroup, Clone)]
struct PotionMaterial {
    #[uniform(1)]
    color: Vec4,

    #[uniform(1)]
    positions: [Vec3; 8],
}

impl Material for PotionMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
