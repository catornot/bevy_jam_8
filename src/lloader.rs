use avian3d::{
    parry::utils::hashmap::{Entry, HashMap},
    prelude::*,
};
pub use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    math::NormedVectorSpace as _,
};
use std::{io::Error as IoError, ops::Sub};

use crate::{prelude::*, wood_grain::WoodGrainMaterial};

const GRID_SIZE: f32 = 5.;
const WALL_HEIGHT: f32 = 10.;

pub fn lloader_plugin(app: &mut App) {
    app.add_systems(
        OnExit(GlobalStates::AssetLoading),
        (spawn_objects, spawn_walls, ground_scene),
    )
    .init_asset::<LevelData>()
    .init_asset_loader::<LevelDataLoader>();
}

#[derive(AssetCollection, Resource)]
pub struct LevelAssets {
    #[asset(path = "levels/map_default.map")]
    level_data: Handle<LevelData>,
}

#[derive(Debug, Asset, Reflect, Clone)]
pub struct LevelData {
    map: Vec<LevelElement>,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone, Copy, Reflect)]
pub enum LevelElement {
    Cauldron,
    PlayerSpawn,
    Fox,
    Bird,
    Tree,
    Wall,
    Bush,
    Empty,
}
const LEVEL_ELEMENTS: [LevelElement; 8] = [
    LevelElement::Cauldron,
    LevelElement::PlayerSpawn,
    LevelElement::Fox,
    LevelElement::Bird,
    LevelElement::Tree,
    LevelElement::Wall,
    LevelElement::Bush,
    LevelElement::Empty,
];

#[derive(Debug, Default, Reflect)]
pub struct LevelDataLoader;

impl AssetLoader for LevelDataLoader {
    type Asset = LevelData;

    type Settings = ();

    type Error = IoError;

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async {
            let mut buf = String::new();
            reader.read_to_string(&mut buf).await?;

            let elements_map =
                LEVEL_ELEMENTS.map(|e| (format!("{e:?}").chars().next().unwrap(), e));

            let height = buf.lines().count();
            let width = buf
                .lines()
                .map(|line| line.len())
                .enumerate()
                .try_fold(None, |v, (x, w)| {
                    v.unwrap_or(w)
                        .eq(&w)
                        .then_some(Some(w))
                        .ok_or_else(|| format!("invalid width at {}", x + 1))
                })
                .map_err(|err: String| IoError::new(std::io::ErrorKind::InvalidData, err))?
                .unwrap_or_default();

            let data = buf
                .lines()
                .enumerate()
                .flat_map(|(x, line)| line.chars().enumerate().map(move |(y, c)| ((x, y), c)))
                .try_fold(Vec::new(), |mut acc, ((x, y), c)| {
                    acc.push(
                        elements_map
                            .iter()
                            .find_map(|(cmp_c, elem)| c.eq(cmp_c).then_some(*elem))
                            .ok_or_else(|| format!("invalid char at {:?}", (x + 1, y + 1)))?,
                    );

                    Ok(acc)
                })
                .map_err(|err: String| IoError::new(std::io::ErrorKind::InvalidData, err))?;

            Ok(LevelData {
                map: data,
                width,
                height,
            })
        }
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}

pub fn spawn_objects(world: &mut World) {
    let level = world
        .get_resource::<LevelAssets>()
        .expect("failed to get required stuff");
    let level_assets = world
        .get_resource::<Assets<LevelData>>()
        .expect("failed to get required stuff");

    let Some(map) = level_assets.get(level.level_data.id()).cloned() else {
        log::error!("failed to open map");
        return;
    };

    let map = &map;
    for ((x, y), element) in (0..map.width).flat_map(|y| {
        (0..map.height).filter_map(move |x| Some(((x, y), map.map.get(y + x * map.width)?)))
    }) {
        let transform =
            Transform::from_translation(Vec3::new(x as f32 * GRID_SIZE, 0., y as f32 * GRID_SIZE));
        match element {
            LevelElement::Cauldron => {
                let s = world.register_system_cached(crate::cauldron::spawn_cauldron);
                if let Err(err) = world.run_system_with(s, transform) {
                    log::error!("{element:?}: {err}");
                }
            }
            LevelElement::PlayerSpawn => {
                let s = world.register_system_cached(crate::player::spawn_player);
                if let Err(err) = world.run_system_with(s, transform) {
                    log::error!("{element:?}: {err}");
                }
            }
            LevelElement::Fox => {
                let s = world.register_system_cached(crate::fox::spawn_fox);
                if let Err(err) = world.run_system_with(s, transform) {
                    log::error!("{element:?}: {err}");
                }
            }
            LevelElement::Bird => {}
            LevelElement::Tree => {
                let s = world.register_system_cached(crate::tree::spawn_tree);
                if let Err(err) = world.run_system_with(s, transform) {
                    log::error!("{element:?}: {err}");
                }
            }

            LevelElement::Wall => {}
            LevelElement::Bush => {}
            LevelElement::Empty => {}
        }
    }
}

fn ground_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    level: Res<LevelAssets>,
    level_assets: Res<Assets<LevelData>>,
) {
    let Some(map) = level_assets.get(level.level_data.id()) else {
        log::error!("failed to open map");
        return;
    };

    let standard_material = StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("ground.png")),
        ..default()
    };

    let size = (map.width).max(map.height) as f32 * GRID_SIZE * 100.;
    let (width, height) = (map.width, map.height);
    let ground = bsn! {
        #Ground
        Mesh3d({meshes.add(Cuboid::new(size, 1.0, size))})
        MeshMaterial3d::<StandardMaterial>(asset_value(standard_material))
        Collider::cuboid(size, 1.0, size)
        RigidBody::Static
        Transform::from_translation(Vec3::new(height as f32 * GRID_SIZE / 2.,-1.,width as f32 * GRID_SIZE / 2.))
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
    };

    commands.spawn_scene(ground);
}

fn spawn_walls(
    level: Res<LevelAssets>,
    level_assets: Res<Assets<LevelData>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut wood_grain_assets: ResMut<Assets<WoodGrainMaterial>>,
) {
    let Some(map) = level_assets.get(level.level_data.id()) else {
        log::error!("failed to open map");
        return;
    };

    let material =
        MeshMaterial3d::<WoodGrainMaterial>(wood_grain_assets.add(WoodGrainMaterial::default()));
    let mut segments = HashMap::default();

    for pos in
        (0..map.width).flat_map(|y| (0..map.height).map(move |x| UVec2::new(x as u32, y as u32)))
    {
        let cell_shape = get_cell_shape(pos, map);
        let pos = pos.to_array();

        match cell_shape {
            0 | 15 => {}
            1 => add_segment(&mut segments, pos, Vec2::new(0., 0.), Vec2::new(0., -1.)), // left
            2 => add_segment(&mut segments, pos, Vec2::new(1., 0.), Vec2::new(0., 0.)),  // bottom
            3 => {
                add_segment(&mut segments, pos, Vec2::new(1., 0.), Vec2::new(0., -0.2)); // bottom
                add_segment(&mut segments, pos, Vec2::new(0., -0.2), Vec2::new(0., -1.));
                // left
            } // top-left
            4 => add_segment(&mut segments, pos, Vec2::new(1., -1.), Vec2::new(1., 0.)), // rigth
            5 => {
                add_segment(&mut segments, pos, Vec2::new(0., 0.), Vec2::new(0., -1.)); // left
                add_segment(&mut segments, pos, Vec2::new(1., -1.), Vec2::new(1., 0.));
                // right
            } // middle between left and right
            6 => {
                add_segment(&mut segments, pos, Vec2::new(1., -1.), Vec2::new(1., -0.2)); // rigth
                add_segment(&mut segments, pos, Vec2::new(1., -0.2), Vec2::new(0., 0.));
                // bottom
            } // top-right
            7 => {
                add_segment(&mut segments, pos, Vec2::new(1., 0.), Vec2::new(0., 0.)); // bottom
                add_segment(&mut segments, pos, Vec2::new(0., 0.), Vec2::new(0., -1.)); // left
                add_segment(&mut segments, pos, Vec2::new(1., -1.), Vec2::new(1., 0.));
                // right
            } // between left right and bottom
            8 => add_segment(&mut segments, pos, Vec2::new(0., -1.), Vec2::new(1., -1.)), // top
            9 => {
                add_segment(&mut segments, pos, Vec2::new(0., -0.8), Vec2::new(1., -1.)); // top
                add_segment(&mut segments, pos, Vec2::new(0., 0.), Vec2::new(0., -0.8));
                // left
            } // bottom-left? what
            10 => {
                add_segment(&mut segments, pos, Vec2::new(1., 0.), Vec2::new(0., 0.)); // bottom
                add_segment(&mut segments, pos, Vec2::new(0., -1.), Vec2::new(1., -1.));
                // top
            } // middle between top and bottom
            11 => {
                add_segment(&mut segments, pos, Vec2::new(0., 0.), Vec2::new(0., -1.)); // left
                add_segment(&mut segments, pos, Vec2::new(1., 0.), Vec2::new(0., 0.)); // bottom
                add_segment(&mut segments, pos, Vec2::new(0., -1.), Vec2::new(1., -1.));
                // top
            } // between top bottom and left
            12 => {
                add_segment(&mut segments, pos, Vec2::new(0., -1.), Vec2::new(1., -0.8)); // top
                add_segment(&mut segments, pos, Vec2::new(1., -0.8), Vec2::new(1., 0.));
                // right
            } // bottom-right?
            13 => {
                add_segment(&mut segments, pos, Vec2::new(0., -1.), Vec2::new(1., -1.)); // top
                add_segment(&mut segments, pos, Vec2::new(0., 0.), Vec2::new(0., -1.)); // left
                add_segment(&mut segments, pos, Vec2::new(1., -1.), Vec2::new(1., 0.));
                // right
            } // between left right and top
            14 => {
                add_segment(&mut segments, pos, Vec2::new(0., -1.), Vec2::new(1., -1.)); // top
                add_segment(&mut segments, pos, Vec2::new(1., 0.), Vec2::new(0., 0.)); // bottom
                add_segment(&mut segments, pos, Vec2::new(1., -1.), Vec2::new(1., 0.));
                // right
            } // between top bottom and right
            _ => unreachable!(),
        };
    }

    let polylines = assemble_polylines(segments);

    let mut children = Vec::new();

    for mut wall in polylines
        .into_iter()
        .map(|mut polyline| {
            polyline.reverse();
            polyline
        })
        .map(|polyline| (Vec::with_capacity(polyline.len() / 10), polyline))
        .map(|(mut polyline_out, polyline_in)| {
            if polyline_in.len() > 4 {
                douglas_peucker_closed_path(&polyline_in, &mut polyline_out).expect("oh no");
            } else {
                polyline_out.clone_from(&polyline_in);
            }
            polyline_out
        })
    {
        wall.iter_mut().for_each(|v| *v *= Vec2::new(1., -1.));

        for ((mesh, collider), segment) in
            wall.array_windows::<2>()
                .filter(|[l, r]| !l.eq(r))
                .map(|[start, end]| {
                    (
                        (
                            Cuboid::new(start.distance(*end), WALL_HEIGHT, 0.5),
                            Collider::cuboid(start.distance(*end), WALL_HEIGHT, 0.5),
                        ),
                        [
                            start.extend(WALL_HEIGHT / 2.).xzy(),
                            end.extend(WALL_HEIGHT / 2.).xzy(),
                        ],
                    )
                })
        {
            let middle = segment[0].midpoint(segment[1]);
            let direction = (segment[1] - segment[0]).normalize();
            children.push(
                commands
                    .spawn((
                        Mesh3d(meshes.add(mesh)),
                        RigidBody::Static,
                        collider,
                        material.clone(),
                        Transform::from_translation(middle)
                            .with_rotation(Quat::from_rotation_arc(Vec3::X, direction)),
                    ))
                    .id(),
            );
        }
    }

    commands
        .spawn(Transform::from_xyz(0., -1., 0.))
        .add_children(&children);
}

fn cords_to_index([x, y]: [usize; 2], width: usize) -> usize {
    y + x * width
}

pub fn get_cell_shape(pos: UVec2, map: &LevelData) -> usize {
    let is_solid = Some(pos)
        .map(|pos| pos.to_array().map(|x| x as usize))
        .map(|pos| {
            matches!(
                map.map.get(cords_to_index(pos, map.width)),
                Some(LevelElement::Wall)
            )
        })
        .unwrap_or_default();

    if !is_solid {
        find_square_value(map, pos, is_solid)
    } else {
        0
    }
}

fn find_square_value(map: &LevelData, pos: UVec2, is_solid: bool) -> usize {
    [(-1, 0), (0, -1), (1, 0), (0, 1)]
        .into_iter()
        .map(|offset| {
            Some(matches!(
                map.map.get(cords_to_index(
                    [
                        TryInto::<usize>::try_into(pos.x as i32 + offset.0).ok()?,
                        TryInto::<usize>::try_into(pos.y as i32 + offset.1).ok()?,
                    ],
                    map.width,
                ))?,
                LevelElement::Wall
            ))
        })
        .map(|cell| cell.unwrap_or(is_solid))
        .enumerate()
        .map(|(i, flag_pass)| 2usize.pow(i as u32) * flag_pass as usize)
        .sum::<usize>()
}

fn add_segment(segments: &mut HashMap<IVec2, [Vec2; 2]>, pos: [u32; 2], start: Vec2, end: Vec2) {
    let size = Vec2::splat(GRID_SIZE);
    let offset = Vec2::new(pos[0] as f32, -(pos[1] as f32)) * size;
    segments.insert(
        // ceil has been applied in both of the places
        start.mul_add(size, offset).ceil().as_ivec2(),
        [start.mul_add(size, offset), end.mul_add(size, offset)],
    );
}

fn assemble_polylines(mut segments: HashMap<IVec2, [Vec2; 2]>) -> Vec<Vec<Vec2>> {
    let mut multi_polylines = Vec::new();

    let mut boundaries = segments.keys().cloned().collect::<Vec<_>>();

    while !segments.is_empty() {
        let first_key = boundaries
            .pop()
            .unwrap_or_else(|| segments.keys().next().copied().unwrap());

        let first = match segments.entry(first_key) {
            Entry::Occupied(occupied) => {
                let first = *occupied.get();

                occupied.remove_entry();

                first
            }
            // Entry::Vacant(_) => panic!("non closed loop"),
            // this should not happen but it does, this means that marching squares produces overlapping segments
            // from observation this usally happens when 2 polylines are to be built and ??
            Entry::Vacant(_) => continue,
        };

        let mut polyline = vec![first[0], first[1]];

        loop {
            let previous = polyline.last().copied().unwrap();

            let key = previous.ceil().as_ivec2();
            if let Entry::Occupied(entry) = segments.entry(key) {
                polyline.push(entry.get()[1]);

                entry.remove_entry();
                if let Some(index) = boundaries.iter().position(|bound| *bound == first_key) {
                    boundaries.swap_remove(index);
                }
            } else {
                break;
            }
        }

        multi_polylines.push(polyline);
    }

    multi_polylines
}

const EPSILON: f32 = 1.5 * GRID_SIZE;
fn douglas_peucker_closed_path(points: &[Vec2], output: &mut Vec<Vec2>) -> Option<()> {
    if points.last() == points.first() {
        douglas_peucker(&points[..points.len().sub(1)], output)?;
        output.push(*points.last().unwrap());
    } else {
        douglas_peucker(points, output)?;
    }

    Some(())
}

fn douglas_peucker(points: &[Vec2], output: &mut Vec<Vec2>) -> Option<()> {
    if points.len() < 3 {
        output.extend_from_slice(points);
        return Some(());
    }

    let (index, max_distance) = points
        .iter()
        .skip(1)
        .take(points.len().saturating_sub(1))
        .copied()
        .enumerate()
        .filter_map(|(i, point)| {
            Some((
                i + 1,
                perpendicular_distance([*points.first()?, *points.last()?], point),
            ))
        })
        .fold(
            (0, f32::MIN),
            |(index_max, max_distance), (index, distance)| {
                if max_distance < distance {
                    (index, distance)
                } else {
                    (index_max, max_distance)
                }
            },
        );

    if max_distance > EPSILON {
        douglas_peucker_closed_path(points.get(..=index).expect("index oob"), output)?;
        douglas_peucker_closed_path(points.get(index..).expect("first index oob"), output)?;
    } else if !points.is_empty() {
        output.push(*points.first()?);
        output.push(*points.last()?);
    }

    Some(())
}

fn perpendicular_distance(line: [Vec2; 2], off_point: Vec2) -> f32 {
    // magic
    (line[1] - line[0])
        .perp_dot(line[0] - off_point)
        .norm()
        .abs()
        / (line[1] - line[0]).norm()
}
