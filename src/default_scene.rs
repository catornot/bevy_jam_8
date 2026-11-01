use avian3d::prelude::*;
use bevy::prelude::*;

pub fn default_scene_plugin(app: &mut App) {
    app.add_systems(Startup, (scene.spawn(), ground));
}

fn ground(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(100.0, 1.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(asset_server.load("ground.png")),
            ..default()
        })),
        Collider::cuboid(100.0, 1.0, 100.0),
        RigidBody::Static,
    ));
}

/// set up a simple 3D scene
fn scene() -> impl SceneList {
    bsn_list! {
        #Cube
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, 0.0)
        Collider::cuboid(1.0, 1.0,1.0)
        RigidBody::Static
        --
        PointLight {
            shadow_maps_enabled: true,
        }
        Transform::from_xyz(4.0, 8.0, 4.0)
        --
        // Camera3d
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
    }
}
