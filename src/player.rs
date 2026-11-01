use avian3d::{parry::glamx::approx::RelativeEq, prelude::*};
use bevy::{
    asset::RenderAssetUsages,
    ecs::{lifecycle::HookContext, relationship::Relationship, world::DeferredWorld},
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
use std::f32::consts::PI;

use crate::pickups::HighlightPickup;
use crate::{
    camera::{CameraDistance, CameraZoom, CharacterControllerCamera, CharacterControllerCameraOf},
    pickups::PickupRayCaster,
    prelude::*,
};

#[derive(Component, Default, Clone, Reflect)]
pub struct Player;

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

#[derive(Component, Clone)]
pub struct MovingToBackpack {
    offset: Dir3,
    starting_distance: Option<f32>,
}

impl Default for MovingToBackpack {
    fn default() -> Self {
        Self {
            offset: Dir3::Y,
            starting_distance: None,
        }
    }
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
            )
                .run_if(in_state(GlobalStates::Game)),
        )
        .add_systems(OnExit(GlobalStates::Game), release_cursor)
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
                CharacterController::default()
                WorldAssetRoot({player_assets.player_scene.clone()})
                RigidBody::Kinematic
                Collider::cylinder(0.7, 1.6)
                Transform::from_xyz(0.0, 100.0, 0.0).with_scale(Vec3::splat(2.))
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

pub fn deposit_in_backpack(
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

fn move_to_backpack(
    mut commands: Commands,
    moving: Query<(&mut Transform, &Backpacked, &mut MovingToBackpack, Entity), Without<Backpack>>,
    targets: Query<&Transform, With<Backpack>>,
    time: Res<Time>,
) {
    for (mut transform, backpacked, mut moving, ent) in moving {
        let Ok(parent) = targets.get(backpacked.get()) else {
            continue;
        };

        if moving.starting_distance.is_none() {
            moving
                .starting_distance
                .replace(transform.translation.distance(parent.translation));
        }
        let Some(starting_distance) = moving.starting_distance else {
            continue;
        };

        let distance = transform.translation.distance(parent.translation);
        let target = parent.translation + moving.offset.as_vec3();
        transform
            .translation
            .x
            .smooth_nudge(&target.x, 10f32.ln(), time.delta_secs() * 2.);
        transform.translation.y.smooth_nudge(
            &(target.y + ((distance / starting_distance) * PI).sin()),
            10f32.ln(),
            time.delta_secs() * 2.,
        );
        transform
            .translation
            .z
            .smooth_nudge(&target.z, 10f32.ln(), time.delta_secs() * 2.);

        if target.relative_eq(&transform.translation, f32::EPSILON, 0.01)
            && let Ok(mut cmds) = commands.get_entity(ent)
        {
            cmds.remove::<MovingToBackpack>();
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
