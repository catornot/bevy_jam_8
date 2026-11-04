pub use bevy::prelude::*;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::{interactable::Interactable, prelude::*};

const SHADER_ASSET_PATH: &str = "shaders/wood_grain.wesl";

pub fn wood_grain_plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<WoodGrainMaterial>::default())
        .add_systems(OnEnter(GlobalStates::Game), spawn_wood);
}

pub type WoodGrainMaterial = ExtendedMaterial<StandardMaterial, WoodGrainExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct WoodGrainExtension {}

impl MaterialExtension for WoodGrainExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

fn spawn_wood(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WoodGrainMaterial>>,
) {
    let material = WoodGrainMaterial::default();
    commands.spawn_scene(bsn! {
        #Wood
        Transform::from_xyz(10., 1.5, 10.)
        Interactable
        Transform::from_translation(Vec3::new(0., 0.25, 0.))
        Mesh3d({meshes.add(Cuboid::new(5., 2., 3.))})
        MeshMaterial3d<WoodGrainMaterial>({materials.add(material)})
    });
}
