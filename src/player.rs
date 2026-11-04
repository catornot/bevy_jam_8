use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    ecs::change_detection::Tick,
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
    backpack::{Backpack, BackpackAttachment, PickupItem},
    bounding_polyhedron::BoundingPolyhedronMarker,
    camera::{CameraDistance, CameraZoom, CharacterControllerCamera, CharacterControllerCameraOf},
    interactable::InteractableRayCaster,
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
                rotate_player_model_with_camera,
                add_model_marker,
            )
                .run_if(in_state(GlobalStates::Game)),
        )
        .add_systems(OnExit(GlobalStates::Game), release_cursor)
        .add_observer(add_pending_model_marker);
}

pub fn spawn_player(
    mut commands: Commands,
    mut assets: ResMut<AssetServer>,
    player_assets: Res<PlayerAssets>,
) {
    // spawn player
    let player = commands
        .spawn_scene(bsn! {
                # Player
                Player
                CharacterController {
                    crouch_height: 0.5,
                    mantle_height: 0.5,
                    climb_pull_up_height: 0.5,
                    standing_view_height: 0.5,
                }
                BoundingPolyhedronMarker::new(0.2).swizzle()
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
    let camera = commands
        .spawn((
            CameraDistance(0.),
            RayCaster::default().with_direction(Dir3::NEG_Z),
            Camera3d::default(),
            InteractableRayCaster,
            // Enable the optional builtin camera controller
            CharacterControllerCameraOf::new(player),
        ))
        .id();

    // spawn audio listener
    let listener = SpatialListener::new(0.2);
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        listener.clone(),
        children![
            // left ear indicator
            (Transform::from_translation(listener.left_ear_offset),),
            // right ear indicator
            (Transform::from_translation(listener.right_ear_offset),)
        ],
        ChildOf(camera),
    ));

    commands.spawn_scene(cursor_scene(camera, assets.as_mut()));
}

fn cursor_scene(target_camera: Entity, assets: &mut AssetServer) -> impl Scene {
    let size = Extent3d {
        width: 100,
        height: 100,
        depth_or_array_layers: 1,
    };
    let background = assets.add(Image::new(
        size,
        bevy::render::render_resource::TextureDimension::D2,
        Vec::from_iter(
            (0..size.width)
                .flat_map(|x| (0..size.height).map(move |y| (x as i64, y as i64)))
                .map(|(x, y)| {
                    if (x - size.width as i64 / 2).pow(2) + (y - size.height as i64 / 2).pow(2)
                        <= 45i64.pow(2)
                    {
                        255
                    } else {
                        0
                    }
                })
                .flat_map(|c| [0, 0, 0, c]),
        ),
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
    children: Query<(&ChildOf, Entity, Ref<PendingModelMarker>)>,
    players: Query<(), With<Player>>,
    mut commands: Commands,
) {
    for (child_of, ent, pending_marker) in children {
        // wait one frame for stuff to settle
        if !pending_marker.is_added_after(Tick::new(1)) {
            continue;
        }

        if let Ok(mut cmds) = commands.get_entity(ent) {
            if players.get(child_of.parent()).is_ok() {
                cmds.insert(PlayerModel(child_of.parent()));
            };
            cmds.remove::<PendingModelMarker>();
        }

        if let Ok(mut cmds) = commands.get_entity(child_of.parent()) {
            cmds.insert(BackpackAttachment::new(ent));
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
