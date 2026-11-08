use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    light::Skybox,
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

use crate::prelude::*;

pub fn default_scene_plugin(app: &mut App) {
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb_u8(210, 220, 240),
        brightness: 1.0,
        ..default()
    })
    .add_observer(add_skybox_to_camera);
}

fn add_skybox_to_camera(trigger: On<Add<Camera>>, mut commands: Commands, cubemap: Res<Cubemap>) {
    if let Ok(mut camera_cmds) = commands.get_entity(trigger.entity) {
        camera_cmds.insert((
            Msaa::Off,
            TemporalAntiAliasing::default(),
            ScreenSpaceAmbientOcclusion::default(),
            Skybox {
                brightness: 1000.0,
                image: Some(cubemap.image_handle.clone()),
                ..Default::default()
            },
        ));
    }
}

#[derive(Resource)]
pub struct Cubemap {
    image_handle: Handle<Image>,
}

impl AssetCollection for Cubemap {
    fn create(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let image_handle = asset_server.load("skybox.png");
        let mut images = world.resource_mut::<Assets<Image>>();
        let mut image = images
            .get_mut(&image_handle)
            .expect("cubemap image should be loaded by now");

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

        Self { image_handle }
    }

    fn load(world: &mut World) -> Vec<UntypedHandle> {
        let asset_server = world.resource::<AssetServer>();

        vec![asset_server.load::<Image>("skybox.png").untyped()]
    }
}
