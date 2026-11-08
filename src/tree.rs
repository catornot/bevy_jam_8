use avian3d::prelude::*;
pub use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::{
    bounding_polyhedron::BoundingPolyhedronMarker, prelude::*, wood_grain::WoodGrainMaterial,
};

pub fn tree_plugin(app: &mut App) {
    app.add_observer(edit_tree_material);
}

#[derive(Component, Default, Clone, Reflect)]
#[require(CollisionLayers::new(GameLayer::Ground, LayerMask::ALL))]
pub struct Tree;

#[derive(AssetCollection, Resource)]
pub struct TreeAssets {
    #[asset(path = "tree.glb#Scene0")]
    tree_scene: Handle<WorldAsset>,
}

pub fn spawn_tree(
    In(transform): In<Transform>,
    mut commands: Commands,
    tree_assets: Res<TreeAssets>,
) {
    let tree = bsn! {
        #Tree
        Tree
        WorldAssetRoot({tree_assets.tree_scene.clone()})
        BoundingPolyhedronMarker::new(1.).with_collier()
        RigidBody::Static
        transform.with_translation(transform.translation.with_y(transform.translation.y - 0.7))
    };

    commands.spawn_scene(tree);
}

fn edit_tree_material(
    trigger: On<WorldInstanceReady>,
    mut commands: Commands,
    entities: Query<(), With<Tree>>,
    children: Query<&Children>,
    mut wood_grain_material_assets: ResMut<Assets<WoodGrainMaterial>>,
) {
    let entity = trigger.entity;
    if !entities.contains(trigger.entity) {
        return;
    }

    let mesh_material = MeshMaterial3d::<WoodGrainMaterial>(
        wood_grain_material_assets.add(WoodGrainMaterial::default()),
    );

    for descendant in children.iter_descendants(entity) {
        commands
            .entity(descendant)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(mesh_material.clone());
    }
}
