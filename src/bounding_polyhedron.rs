use avian3d::parry::shape::ConvexPolyhedron;
use bevy::{mesh::VertexAttributeValues, prelude::*};

pub fn bounding_polyhedron_plugin(app: &mut App) {
    app.register_type::<BoundingPolyhedronMarker>()
        .add_systems(Update, calculate_bounding_polyhedron);
}

/// the bounding polyhedron for the model of an entity, used for by the backpack system
#[derive(Component, Clone)]
pub struct BoundingPolyhedron(ConvexPolyhedron);

impl BoundingPolyhedron {
    pub fn get(&self) -> &ConvexPolyhedron {
        &self.0
    }
}

/// the f32 is the scale
#[derive(Component, Clone, Reflect)]
pub struct BoundingPolyhedronMarker(f32);

impl Default for BoundingPolyhedronMarker {
    fn default() -> Self {
        Self(1.)
    }
}

impl BoundingPolyhedronMarker {
    pub fn new(scale: f32) -> Self {
        Self(scale.max(0.))
    }
}

fn calculate_bounding_polyhedron(
    mut commands: Commands,
    roots: Query<(Entity, &BoundingPolyhedronMarker), Without<BoundingPolyhedron>>,
    children: Query<&Children>,
    meshes: Query<&Mesh3d>,
    assets: Res<Assets<Mesh>>,
) {
    for (root, &BoundingPolyhedronMarker(scale)) in roots {
        let mut queue = vec![root];
        let mut vertices = Vec::new();

        while let Some(node) = queue.pop().and_then(|next_ent| {
            children
                .get(next_ent)
                .ok()
                .map(|children| Ok((children, meshes.get(next_ent).ok())))
                .or_else(|| meshes.get(next_ent).ok().map(Err))
        }) {
            if let Some(mesh) = match node {
                Ok((children, mesh)) => {
                    queue.extend(children.iter());
                    mesh
                }
                Err(mesh) => Some(mesh),
            }
            .and_then(|mesh| assets.get(mesh))
            {
                let Ok(VertexAttributeValues::Float32x3(values)) =
                    mesh.try_attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    continue;
                };
                vertices.extend(values.iter().copied().map(Vec3::from).map(|v| v.xzy()));
            }
        }

        if let Ok(mut root_cmds) = commands.get_entity(root) {
            // because once the player is loaded for the first time, it would still be missing the gltf meshes
            let Some(convex_polyhedron) = ConvexPolyhedron::from_convex_hull(&vertices)
                .and_then(|poly| poly.scaled(Vec3::splat(scale)))
            else {
                continue;
            };
            root_cmds.insert(BoundingPolyhedron(convex_polyhedron));
        }
    }
}
