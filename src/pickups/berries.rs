use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{cauldron::CauldronEffect, pickups::Pickup, prelude::*};

#[derive(Resource)]
pub struct BerryAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Default, Clone)]
#[require(Pickup)]
pub struct Berry;

pub fn berries_plugin(_: &mut App) {}

impl AssetCollection for BerryAssets {
    fn create(world: &mut World) -> Self {
        BerryAssets {
            mesh: world
                .resource_mut::<Assets<Mesh>>()
                .add(Cone::new(0.1, 0.2)),
            material: world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgb(1., 0.1, 0.1),
                    emissive: LinearRgba {
                        red: 1.,
                        green: 0.1,
                        blue: 0.1,
                        alpha: 0.5,
                    },
                    ..default()
                }),
        }
    }

    fn load(_: &mut World) -> Vec<UntypedHandle> {
        Vec::new()
    }
}

pub fn berry_scene(BerryAssets { mesh, material }: &BerryAssets) -> impl Scene {
    let mesh = mesh.clone();
    let material = material.clone();
    bsn! {
        # Berry
        Berry
        Mesh3d(mesh)
        MeshMaterial3d<StandardMaterial>(material)
        Collider::cone(0.1, 0.2)
        CauldronEffect(Vec3::new(0., 0.3, 0.1))
    }
}
