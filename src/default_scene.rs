use avian3d::prelude::*;
use bevy::{
    light::Skybox, prelude::*, render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

pub fn default_scene_plugin(app: &mut App) {
    app.add_systems(Startup, (scene, skybox_startup))
        .add_systems(Update, skybox_update);
}

fn scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_scene_list(other_scene());
    commands.spawn_scene(ground_scene(100., &asset_server));
}

fn ground_scene(size: f32, asset_server: &AssetServer) -> impl Scene {
    let standard_material = StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("ground.png")),
        ..default()
    };

    bsn! {
        Mesh3d(asset_value(Cuboid::new(size, 1.0, size)))
        MeshMaterial3d::<StandardMaterial>(asset_value(standard_material))
        Collider::cuboid(size, 1.0, size)
        RigidBody::Static
    }
}

fn other_scene() -> impl SceneList {
    bsn_list! {
        #Cube
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, 0.0)
        Collider::cuboid(1.0, 1.0,1.0)
        RigidBody::Static
        --
        #PointLight
        PointLight {
            shadow_maps_enabled: true,
        }
        Transform::from_xyz(4.0, 8.0, 4.0)
    }
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
        image_handle,
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
