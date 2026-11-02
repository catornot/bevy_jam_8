use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    gltf::GltfSceneName,
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::render_resource::Extent3d,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_ahoy::{CharacterLook, prelude::*};
use bevy_enhanced_input::{
    prelude::*,
    preset::{axial, cardinal},
};

use crate::{
    backpack::{Backpack, PickupItem},
    bounding_polyhedron::{BoundingPolyhedron, BoundingPolyhedronMarker},
    camera::{CameraDistance, CameraZoom, CharacterControllerCamera, CharacterControllerCameraOf},
    pickups::PickupRayCaster,
    prelude::*,
};

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
        .add_systems(OnEnter(GlobalStates::Game), (spawn_player, capture_cursor))
        .add_systems(
            Update,
            (
                capture_cursor.run_if(input_just_pressed(KeyCode::Delete)),
                release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                model_visibility,
                debug_bounding_polyhedron,
                rotate_player_model_with_camera,
                add_model_marker,
            )
                .run_if(in_state(GlobalStates::Game)),
        )
        .add_systems(OnExit(GlobalStates::Game), release_cursor)
        .add_observer(add_pending_model_marker);
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
                BoundingPolyhedronMarker::new(0.2)
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
    mut players: Query<(Entity, &mut Backpack), With<Player>>,
    mut commands: Commands,
) {
    for (child_of, ent) in children {
        let Ok((player_ent, mut backpack)) = players.get_mut(child_of.parent()) else {
            return;
        };

        if let Ok(mut cmds) = commands.get_entity(ent) {
            cmds.insert(PlayerModel(player_ent));
            backpack.attachment_entity.replace(ent);
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

fn debug_bounding_polyhedron(
    bounding_polyhedrons: Query<(&Transform, &BoundingPolyhedron)>,
    mut gizmos: Gizmos,
) {
    for (transform, bounding_polyhedron) in bounding_polyhedrons {
        let points = bounding_polyhedron.get().points();

        for edge in bounding_polyhedron.get().edges() {
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

fn rotate_player_model_with_camera(
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
