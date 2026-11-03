use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::GlobalStates;

pub fn fox_plugin(app: &mut App) {
    app.register_type::<Fox>()
        .add_systems(OnEnter(GlobalStates::Game), spawn_fox);
}

#[derive(AssetCollection, Resource)]
pub struct FoxAssets {
    #[asset(path = "Fox.glb#Scene0")]
    fox_scene: Handle<WorldAsset>,
}

#[derive(Component, Default, Clone, Reflect)]
pub struct Fox;

pub fn spawn_fox(
    mut command: Commands,
    fox_assets: Res<FoxAssets>,
) {
        command.spawn_scene(bsn! {
                #Fox
                WorldAssetRoot({fox_assets.fox_scene.clone()})
                Transform::from_xyz(0.0, 2.0, 0.0).with_scale(Vec3::splat(0.1))
        });
}
