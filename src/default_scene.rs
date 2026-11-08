use avian3d::prelude::*;
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    light::Skybox,
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

use crate::{fox, prelude::*, wood_grain::WoodGrainMaterial};

pub fn default_scene_plugin(app: &mut App) {
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb_u8(210, 220, 240),
        brightness: 1.0,
        ..default()
    })
    .add_systems(
        OnEnter(GlobalStates::Game),
        (scene, edit_tree_material, fox::spawn_fox).chain(),
    )
    .add_observer(add_skybox_to_camera);
}

fn scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scene_assets: Res<SceneAssets>,
    mut materials: ResMut<Assets<WoodGrainMaterial>>,
) {
    commands.spawn_scene_list(other_scene(scene_assets, &mut materials));
    commands.spawn_scene_list(ground_scene(100., &asset_server));
}

fn ground_scene(size: f32, asset_server: &AssetServer) -> impl SceneList {
    let standard_material = StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("ground.png")),
        ..default()
    };

    let wall_colour = Color::srgb_u8(80, 65, 40);

    bsn_list! {
        #Ground
        Mesh3d(asset_value(Cuboid::new(size, 1.0, size)))
        MeshMaterial3d::<StandardMaterial>(asset_value(standard_material))
        Collider::cuboid(size, 1.0, size)
        RigidBody::Static
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
        --
        Name("Wall+Z")
        Mesh3d(asset_value(Cuboid::new(size, 10.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(wall_colour))
        Transform::from_xyz(0., 0., size/2.)
        Collider::cuboid(size, 10.0, 1.0)
        RigidBody::Static
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
        --
        Name("Wall+X")
        Mesh3d(asset_value(Cuboid::new(1.0, 10.0, size)))
        MeshMaterial3d::<StandardMaterial>(asset_value(wall_colour))
        Transform::from_xyz(size/2., 0., 0.)
        Collider::cuboid(1.0, 10.0, size)
        RigidBody::Static
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
        --
        Name("Wall-Z")
        Mesh3d(asset_value(Cuboid::new(size, 10.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(wall_colour))
        Transform::from_xyz(0., 0., -size/2.)
        Collider::cuboid(size, 10.0, 1.0)
        RigidBody::Static
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
        --
        Name("Wall-X")
        Mesh3d(asset_value(Cuboid::new(1.0, 10.0, size)))
        MeshMaterial3d::<StandardMaterial>(asset_value(wall_colour))
        Transform::from_xyz(-size/2., 0., 0.)
        Collider::cuboid(1.0, 10.0, size)
        RigidBody::Static
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
    }
}

#[derive(Component, Default, Clone, Reflect)]
#[require(CollisionLayers::new(GameLayer::Default, LayerMask::ALL))]
pub struct Tree;

#[derive(AssetCollection, Resource)]
pub struct SceneAssets {
    #[asset(path = "tree.glb#Scene0")]
    tree_scene: Handle<WorldAsset>,
}

fn edit_tree_material_helper(
    commands: &mut Commands,
    entity: Entity,
    children: Query<&Children>,
    mesh_material: &MeshMaterial3d<
        bevy::pbr::ExtendedMaterial<StandardMaterial, crate::wood_grain::WoodGrainExtension>,
    >,
) {
    for descendant in children.iter_descendants(entity) {
        commands
            .entity(descendant)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(mesh_material.clone());
        edit_tree_material_helper(commands, descendant, children, mesh_material);
    }
}

fn edit_tree_material(
    mut commands: Commands,
    entities: Query<Entity, With<Tree>>,
    children: Query<&Children>,
    mut wood_grain_material_assets: ResMut<Assets<WoodGrainMaterial>>,
) {
    let material = WoodGrainMaterial::default();
    let mesh_material =
        MeshMaterial3d::<WoodGrainMaterial>(wood_grain_material_assets.add(material));
    for entity in entities {
        edit_tree_material_helper(&mut commands, entity, children, &mesh_material);
    }
}

fn other_scene(
    scene_assets: Res<SceneAssets>,
    materials: &mut ResMut<Assets<WoodGrainMaterial>>,
) -> impl SceneList {
    let material = WoodGrainMaterial::default();
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
        --
        #Tree
        Tree
        WorldAssetRoot({scene_assets.tree_scene.clone()})
        MeshMaterial3d<WoodGrainMaterial>({materials.add(material)})
    }
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
