use avian3d::prelude::*;
use bevy::{
    light::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

pub fn default_scene_plugin(app: &mut App) {
    app.add_systems(Startup, (skybox_startup, ground, scene.spawn()))
        .add_systems(Update, skybox_update);
}

fn ground(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size: f32 = 500.0;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size, 1.0, size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(asset_server.load("ground.png")),
            ..default()
        })),
        Collider::cuboid(size, 1.0, size),
        RigidBody::Static,
    ));
}

#[derive(Resource)]
struct Cubemap {
    image_handle: Handle<Image>,
    is_updated: bool,
}

fn skybox_startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image_handle: Handle<Image> = asset_server.load("skybox.png");

    commands.spawn(Skybox {
        brightness: 1000.0,
        ..Default::default()
    });

    commands.insert_resource(Cubemap {
        image_handle: image_handle,
        is_updated: false,
    });
}

fn skybox_update(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_updated && asset_server.load_state(&cubemap.image_handle).is_loaded() {
        let mut image = images.get_mut(&cubemap.image_handle).unwrap();
        if image.texture_descriptor.array_layer_count() == 1 {
            let layers = image.height() / image.width();
            image
                .reinterpret_stacked_2d_as_array(layers)
                .expect("asset should be 2d texture and height will always be evenly divisible with the given layers");
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        // there's only one skybox anyway
        for mut skybox in &mut skyboxes {
            skybox.image = Some(cubemap.image_handle.clone());
        }

        cubemap.is_updated = true;
    }
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
