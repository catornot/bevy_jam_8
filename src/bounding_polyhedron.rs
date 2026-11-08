use avian3d::{parry::shape::ConvexPolyhedron, prelude::*};
use bevy::{mesh::VertexAttributeValues, prelude::*};

pub fn bounding_polyhedron_plugin(app: &mut App) {
    app.register_type::<BoundingPolyhedronMarker>()
        .register_type::<BoundingPolyhedronSkipMarker>()
        .add_systems(
            Update,
            (calculate_bounding_polyhedron, debug_bounding_polyhedron),
        );
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
pub struct BoundingPolyhedronMarker(f32, bool, bool);

#[derive(Component, Default, Clone, Reflect)]
pub struct BoundingPolyhedronSkipMarker;

impl Default for BoundingPolyhedronMarker {
    fn default() -> Self {
        Self(1., false, false)
    }
}

impl BoundingPolyhedronMarker {
    pub fn new(scale: f32) -> Self {
        Self(scale.max(0.), false, false)
    }
    pub fn swizzle(self) -> Self {
        Self(self.0, true, self.2)
    }
    pub fn with_collier(self) -> Self {
        Self(self.0, self.1, true)
    }
}

fn calculate_bounding_polyhedron(
    mut commands: Commands,
    roots: Query<(Entity, &BoundingPolyhedronMarker), Without<BoundingPolyhedron>>,
    children: Query<&Children>,
    meshes: Query<&Mesh3d, Without<BoundingPolyhedronSkipMarker>>,
    assets: Res<Assets<Mesh>>,
) {
    for (root, &BoundingPolyhedronMarker(scale, swizzle, as_collider)) in roots {
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
                vertices.extend(
                    values
                        .iter()
                        .copied()
                        .map(Vec3::from)
                        .map(|v| if swizzle { v.xzy() } else { v }),
                );
            }
        }

        if let Ok(mut root_cmds) = commands.get_entity(root) {
            // because once the player is loaded for the first time, it would still be missing the gltf meshes
            let Some(convex_polyhedron) = ConvexPolyhedron::from_convex_hull(&vertices)
                .and_then(|poly| poly.scaled(Vec3::splat(scale)))
            else {
                continue;
            };

            if as_collider {
                let verts = convex_polyhedron.points();
                root_cmds.insert(Collider::convex_hull(verts).expect("this already succeeded"));
            } else {
                root_cmds.insert(BoundingPolyhedron(convex_polyhedron));
            }
        }
    }
}

fn debug_bounding_polyhedron(
    bounding_polyhedrons: Query<(&Transform, &BoundingPolyhedron)>,
    mut gizmos: Gizmos,
) {
    for (transform, bounding_polyhedron) in bounding_polyhedrons {
        let points = bounding_polyhedron.get().points();

        for edge in bounding_polyhedron.get().edges() {
            let a = points[edge.vertices[0] as usize];
            let b = points[edge.vertices[1] as usize];

            let transform = transform.with_scale(Vec3::splat(1.));
            gizmos.line(
                transform.transform_point(a),
                transform.transform_point(b),
                Color::srgb(1., 0., 0.),
            );
        }
    }
}
