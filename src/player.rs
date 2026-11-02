use avian3d::{
    parry::{self, query::RayCast, shape::ConvexPolyhedron},
    prelude::*,
};
use bevy::{
    asset::RenderAssetUsages,
    ecs::{lifecycle::HookContext, relationship::Relationship, world::DeferredWorld},
    gltf::GltfSceneName,
    input::common_conditions::input_just_pressed,
    mesh::VertexAttributeValues,
    prelude::*,
    render::render_resource::Extent3d,
    window::{CursorGrabMode, CursorOptions},
    world_serialization::WorldInstance,
};
use bevy_ahoy::{CharacterLook, prelude::*};
use bevy_enhanced_input::{
    prelude::*,
    preset::{axial, cardinal},
};
use rand::RngExt;
use std::f32::consts::PI;

use crate::{
    camera::{CameraDistance, CameraZoom, CharacterControllerCamera, CharacterControllerCameraOf},
    pickups::PickupRayCaster,
    prelude::*,
};
use crate::{pickups::HighlightPickup, rand::Rand};

#[derive(Component, Default, Clone, Reflect)]
pub struct Player;

#[derive(Component, Default, Clone, Reflect)]
pub struct PendingModelMarker;

#[derive(Component, Clone, Reflect)]
#[relationship(relationship_target = PlayerModelTarget)]
pub struct PlayerModel(Entity);

#[derive(Component, Clone, Reflect)]
#[relationship_target(relationship = PlayerModel)]
pub struct PlayerModelTarget(Entity);

#[derive(Component, Clone)]
pub struct PlayerBoundingPolyhedron(ConvexPolyhedron);

#[derive(Debug, InputAction, Reflect)]
#[action_output(bool)]
pub struct PickupItem;

#[derive(Component, Default, Clone, Reflect)]
#[relationship_target(relationship = Backpacked)]
pub struct Backpack {
    #[relationship]
    items: Vec<Entity>,
}

#[derive(Component, Clone, Reflect)]
#[relationship(relationship_target = Backpack)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
#[require(MovingToBackpack)]
pub struct Backpacked(Entity);

impl Backpacked {
    pub fn on_add(mut world: DeferredWorld<'_>, ctx: HookContext) {
        let mut commands = world.commands();
        let Ok(mut ent_cmds) = commands.get_entity(ctx.entity) else {
            return;
        };
        ent_cmds.insert(RigidBodyDisabled);
        ent_cmds.insert(ColliderDisabled);
    }

    pub fn on_remove(mut world: DeferredWorld<'_>, ctx: HookContext) {
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

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "hedgehog.glb#Scene0")]
    player_scene: Handle<WorldAsset>,
}

#[derive(Component, Default, Clone)]
struct PlayerInput;

pub fn player_plugin(app: &mut App) {
    app.add_input_context::<PlayerInput>()
        .register_type::<Player>()
        .register_type::<Backpack>()
        .register_type::<Backpacked>()
        .register_type::<PickupItem>()
        .add_systems(OnEnter(GlobalStates::Game), (spawn_player, capture_cursor))
        .add_systems(
            Update,
            (
                capture_cursor.run_if(input_just_pressed(KeyCode::Delete)),
                release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                model_visibility,
                move_to_backpack,
                calculate_player_bounding_polyhedron,
                calculate_backpacking_info,
                debug_bounding_polyhedron,
                rotate_player_children_with_camera,
                add_model_marker,
            )
                .run_if(in_state(GlobalStates::Game)),
        )
        .add_systems(OnExit(GlobalStates::Game), release_cursor)
        .add_observer(add_pending_model_marker)
        .add_observer(deposit_in_backpack);
}

pub fn spawn_player(
    mut command: Commands,
    mut assets: ResMut<AssetServer>,
    player_assets: Res<PlayerAssets>,
) {
    // spawn player
    let player = command
        .spawn_scene(bsn! {
                # Player
                Player
                CharacterController {
                    crouch_height: 0.5,
                    mantle_height: 0.5,
                    climb_pull_up_height: 0.5,
                    standing_view_height: 0.5,
                }
                WorldAssetRoot({player_assets.player_scene.clone()})
                RigidBody::Kinematic
                Collider::cylinder(0.2, 0.5)
                Transform::from_xyz(0.0, 100.0, 0.0).with_scale(Vec3::splat(1.))
                PlayerInput
                Actions<PlayerInput> [
                    Action::<Movement>::new()
                    DeadZone::default()
                    Bindings [
                        {cardinal::wasd_keys()}
                        --
                        {axial::left_stick()}
                    ]
                    --
                    Action::<Jump>::new()
                    Press::default()
                    Bindings [
                        Binding (KeyCode::Space)
                        --
                        Binding (GamepadButton::South)
                    ]
                    --
                    Action::<Crouch>::new()
                    ~Hold::new(0.0)
                    Bindings [
                        Binding(KeyCode::ControlLeft)
                        --
                        Binding(GamepadButton::East)
                    ]
                    --
                    Action::<RotateCamera>::new()
                    ~Scale::splat(0.04)
                    DeadZone::default()
                    Bindings [
                        Binding::mouse_motion()
                        --
                        {axial::right_stick()}
                    ]
                    --
                    Action::<CameraZoom>::new()
                    Bindings [
                        Binding::mouse_wheel()
                        ~SwizzleAxis::YXZ
                        --
                        {cardinal::dpad()}
                    ]
                    --
                    Action::<PickupItem>::new()
                    ~Hold::new(0.0)
                    Bindings [
                        Binding(MouseButton::Left)
                        --
                        Binding(GamepadButton::LeftTrigger)
                    ]
                ]
                PointLight
                Backpack
        })
        .id();

    // spawn camera
    let camera = command
        .spawn((
            CameraDistance(0.),
            RayCaster::default().with_direction(Dir3::NEG_Z),
            Camera3d::default(),
            PickupRayCaster,
            // Enable the optional builtin camera controller
            CharacterControllerCameraOf::new(player),
        ))
        .id();

    command.spawn_scene(cursor_scene(camera, assets.as_mut()));
}

fn cursor_scene(target_camera: Entity, assets: &mut AssetServer) -> impl Scene {
    let background = assets.add(Image::new_fill(
        Extent3d {
            width: 50,
            height: 50,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[0, 0, 0, 255],
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ));

    bsn! {
        # Cursor
        UiTargetCamera(target_camera)
        ImageNode {
            image: { background },
        }
        Node {
            width: Val::Px(10.),
            border: UiRect::all(Val::Px(2.)),
            border_radius: BorderRadius::all(Val::Px(1.)),
            position_type: PositionType::Absolute,
            bottom: Val::Percent(50.),
            right: Val::Percent(50.),
        }
    }
}

fn add_pending_model_marker(trigger: On<Add<GltfSceneName>>, mut commands: Commands) {
    if let Ok(mut cmds) = commands.get_entity(trigger.entity) {
        cmds.insert(PendingModelMarker);
    }
}

fn add_model_marker(
    children: Query<(&ChildOf, Entity), With<PendingModelMarker>>,
    players: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for (child_of, ent) in children {
        let Ok(player_ent) = players.get(child_of.parent()) else {
            return;
        };

        if let Ok(mut cmds) = commands.get_entity(ent) {
            cmds.insert(PlayerModel(player_ent));
        }
    }
}

fn calculate_player_bounding_polyhedron(
    mut commands: Commands,
    roots: Query<Entity, (With<Player>, Without<PlayerBoundingPolyhedron>)>,
    children: Query<&Children>,
    meshes: Query<&Mesh3d>,
    assets: Res<Assets<Mesh>>,
) {
    for root in roots {
        let mut queue = vec![root];
        let mut vertices = Vec::new();

        while let Some(node) = queue.pop().and_then(|next_ent| {
            children
                .get(next_ent)
                .ok()
                .map(|children| Ok((children, meshes.get(next_ent).ok())))
                .or_else(|| meshes.get(next_ent).ok().map(Err))
        }) {
            if let Some(mesh) = match node {
                Ok((children, mesh)) => {
                    queue.extend(children.iter());
                    mesh
                }
                Err(mesh) => Some(mesh),
            }
            .and_then(|mesh| assets.get(mesh))
            {
                let Ok(VertexAttributeValues::Float32x3(values)) =
                    mesh.try_attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    continue;
                };
                vertices.extend(values.iter().copied().map(Vec3::from).map(|v| v.xzy()));
            }
        }

        if let Ok(mut root_cmds) = commands.get_entity(root) {
            // because once the player is loaded for the first time, it would still be missing the gltf meshes
            let Some(convex_polyhedron) = ConvexPolyhedron::from_convex_hull(&vertices)
                .and_then(|poly| poly.scaled(Vec3::splat(0.2)))
            else {
                continue;
            };
            root_cmds.insert(PlayerBoundingPolyhedron(convex_polyhedron));
        }
    }
}

fn model_visibility(
    player_children: Query<(&Children, &CharacterControllerCamera), With<Player>>,
    camera_distance: Query<&CameraDistance>,
    mut all_children: Query<&mut Visibility>,
) {
    for (children, new_vis) in player_children.into_iter().map(|(children, camera_ref)| {
        (
            children,
            camera_distance
                .get(camera_ref.get())
                .map(|distance| {
                    if distance.far_enough() {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                })
                .unwrap_or_default(),
        )
    }) {
        for child in children.into_iter().copied() {
            if let Ok(mut vis) = all_children.get_mut(child) {
                *vis = new_vis
            }
        }
    }
}

fn deposit_in_backpack(
    trigger: On<Fire<PickupItem>>,
    mut commands: Commands,
    for_pickup: Query<Entity, With<HighlightPickup>>,
) {
    if !trigger.value {
        return;
    }

    for ent in for_pickup {
        if let Ok(mut cmds) = commands.get_entity(ent) {
            cmds.insert(Backpacked(trigger.context));
        }
    }
}

fn calculate_backpacking_info(
    mut commands: Commands,
    mut rand: ResMut<Rand>,
    pickups: Query<(&Backpacked, Entity, &Transform), (Without<Backpack>, With<MovingToBackpack>)>,
    targets: Query<(&Transform, &PlayerBoundingPolyhedron), With<Backpack>>,
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

                let result = if let Some(distance) = bounding.0.cast_local_ray(
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
                        .0
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
    targets: Query<(&Transform, Entity, &PlayerModelTarget), With<Backpack>>,
    player_models: Query<Entity, With<PlayerModel>>,
    time: Res<Time>,
) {
    for (mut transform, backpacked, backpacking_info, ent) in moving {
        let Ok((parent, parent_ent, player_model_ref)) = targets.get(backpacked.get()) else {
            continue;
        };

        let distance = transform.translation.distance(parent.translation);
        let target = parent.translation + backpacking_info.offset;
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

            if let Ok(player_model_ent) = player_models.get(player_model_ref.0) {
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

fn debug_bounding_polyhedron(
    bounding_polyhedrons: Query<(&Transform, &PlayerBoundingPolyhedron)>,
    mut gizmos: Gizmos,
) {
    for (transform, bounding_polyhedron) in bounding_polyhedrons {
        let points = bounding_polyhedron.0.points();

        for edge in bounding_polyhedron.0.edges() {
            let a = points[edge.vertices[0] as usize];
            let b = points[edge.vertices[1] as usize];

            gizmos.line(
                transform.translation + a,
                transform.translation + b,
                Color::srgb(1., 0., 0.),
            );
        }
    }
}

fn rotate_player_children_with_camera(
    players: Query<(&CharacterLook, &PlayerModelTarget)>,
    mut player_models: Query<&mut Transform, With<PlayerModel>>,
) {
    for (look, model_ref) in players {
        if let Ok(mut transform) = player_models.get_mut(model_ref.0) {
            // TODO: maybe make this smooth later

            transform.look_to(Dir3::Z.rotate_y(look.yaw), Dir3::Y);
        }
    }
}

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
