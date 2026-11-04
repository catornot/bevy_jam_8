use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

use crate::{
    bounding_polyhedron::{BoundingPolyhedronMarker, BoundingPolyhedronSkipMarker},
    interactable::Interactable,
    prelude::*,
};

const SHADER_ASSET_PATH: &str = "shaders/potion_shader.wesl";

pub fn cauldron_plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<PotionMaterial>::default())
        .add_systems(OnEnter(GlobalStates::Game), spawn_cauldron)
        .add_systems(Update, (move_point, sync_color));
}

#[derive(Component, Clone, FromTemplate)]
pub struct Cauldron(Color);

#[derive(AssetCollection, Resource)]
pub struct CauldronAssets {
    #[asset(path = "cauldron.glb#Scene0")]
    cauldron: Handle<WorldAsset>,
}

fn spawn_cauldron(
    mut commands: Commands,
    cauldron_assets: Res<CauldronAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PotionMaterial>>,
) {
    let material = PotionMaterial {
        color: Vec4::new(0.86644, -0.233887, 0.179498, 0.),
        positions: [Vec2::new(0., 0.5).extend(1.); 4],
    };
    commands.spawn_scene(bsn! {
        #Cauldron
        Cauldron(Color::srgba_u8(0,255,0,255))
        WorldAssetRoot({cauldron_assets.cauldron.clone()})
        Transform::from_xyz(0., 1.5, 10.)
        BoundingPolyhedronMarker::new(0.9)
        RigidBody::Static
        Collider::cylinder(1., 1.)
        Interactable
        Children [
            #LiquidPlane
            Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)).with_translation(Vec3::new(0., 0.25, 0.))
            BoundingPolyhedronSkipMarker
            Mesh3d({meshes.add(Circle::new(0.9))})
            MeshMaterial3d<PotionMaterial>({materials.add(material)})
        ]
    });
}

fn sync_color(
    cauldrons: Query<(&Cauldron, &MeshMaterial3d<PotionMaterial>)>,
    mut materials: ResMut<Assets<PotionMaterial>>,
) {
    for (cauldron, material_handle) in cauldrons {
        if let Some(mut material) = materials.get_mut(material_handle.id()) {
            let color: Oklaba = cauldron.0.into();
            material.color = color.to_vec4();
        }
    }
}

fn move_point(
    query: Query<&MeshMaterial3d<PotionMaterial>>,
    mut materials: ResMut<Assets<PotionMaterial>>,
    time: Res<Time>,
) {
    for material in query {
        if let Some(mut potion) = materials.get_mut(material.id()) {
            potion.positions = potion.positions.map(|mut v| {
                v.x += time.delta_secs();
                if v.x > 1.0 { Vec3::ZERO } else { v }
            });
        }
    }
}

#[derive(Asset, Default, TypePath, AsBindGroup, Debug, Clone)]
struct PotionMaterial {
    #[uniform(1)]
    color: Vec4,

    #[uniform(1)]
    positions: [Vec3; 4],
}

impl Material for PotionMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
