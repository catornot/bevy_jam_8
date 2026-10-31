use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::render_resource::Extent3d,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::{
    prelude::*,
    preset::{axial, cardinal},
};

#[derive(Component, Default, Clone)]
pub struct Player;

#[derive(Component, Default, Clone)]
struct PlayerInput;

pub fn player_plugin(app: &mut App) {
    app.add_input_context::<PlayerInput>()
        .add_systems(Startup, spawn_player)
        .add_systems(
            Update,
            (
                capture_cursor.run_if(input_just_pressed(KeyCode::Delete)),
                release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
            ),
        );
}

pub fn spawn_player(mut commands: Commands, mut assets: ResMut<AssetServer>) {
    // spawn player
    let player = commands
        .spawn_scene(bsn! {
                # Player
                Player
                CharacterController::default()
                RigidBody::Kinematic
                Collider::cylinder(0.7, 1.6)
                Transform::from_xyz(0.0, 100.0, 0.0)
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
                        Binding(GamepadButton::LeftTrigger)
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
                ]
                PointLight
        })
        .id();

    // spawn camera
    let camera = commands
        .spawn((
            Camera3d::default(),
            // Enable the optional builtin camera controller
            CharacterControllerCameraOf::new(player),
        ))
        .id();

    commands.spawn_scene(cursor_scene(camera, assets.as_mut()));
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

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
