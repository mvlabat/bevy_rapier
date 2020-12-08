use crate::physics::{ColliderHandleComponent, RapierConfiguration};
use crate::render::RapierRenderColor;
use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use rapier::dynamics::RigidBodySet;
use rapier::geometry::{ColliderSet, ShapeType};

/// System responsible for attaching a PbrComponents to each entity having a collider.
pub fn create_collider_renders_system(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    configuration: Res<RapierConfiguration>,
    bodies: Res<RigidBodySet>,
    colliders: ResMut<ColliderSet>,
    query: Query<
        (Entity, &ColliderHandleComponent, Option<&RapierRenderColor>),
        Without<Handle<Mesh>>,
    >,
) {
    let ground_color = Color::rgb(
        0xF3 as f32 / 255.0,
        0xD9 as f32 / 255.0,
        0xB1 as f32 / 255.0,
    );

    let palette = [
        Color::rgb(
            0x98 as f32 / 255.0,
            0xC1 as f32 / 255.0,
            0xD9 as f32 / 255.0,
        ),
        Color::rgb(
            0x05 as f32 / 255.0,
            0x3C as f32 / 255.0,
            0x5E as f32 / 255.0,
        ),
        Color::rgb(
            0x1F as f32 / 255.0,
            0x7A as f32 / 255.0,
            0x8C as f32 / 255.0,
        ),
    ];

    let mut icolor = 0;
    for (entity, collider, debug_color) in &mut query.iter() {
        if let Some(collider) = colliders.get(collider.handle()) {
            if let Some(body) = bodies.get(collider.parent()) {
                let default_color = if body.is_static() {
                    ground_color
                } else {
                    icolor += 1;
                    palette[icolor % palette.len()]
                };

                let shape = collider.shape();

                let color = debug_color
                    .map(|c| Color::rgb(c.0, c.1, c.2))
                    .unwrap_or(default_color);

                let mesh = match shape.shape_type() {
                    #[cfg(feature = "dim3")]
                    ShapeType::Cuboid => Mesh::from(shape::Cube { size: 1.0 }),
                    #[cfg(feature = "dim2")]
                    ShapeType::Cuboid => Mesh::from(shape::Quad {
                        size: Vec2::new(2.0, 2.0),
                        flip: false,
                    }),
                    ShapeType::Ball => Mesh::from(shape::Icosphere {
                        subdivisions: 2,
                        radius: 1.0,
                    }),
                    #[cfg(feature = "dim2")]
                    ShapeType::Trimesh => {
                        let mut mesh =
                            Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
                        let trimesh = shape.as_trimesh().unwrap();
                        mesh.set_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            VertexAttributeValues::from(
                                trimesh
                                    .vertices()
                                    .iter()
                                    .map(|vertice| [vertice.x, vertice.y])
                                    .collect::<Vec<_>>(),
                            ),
                        );
                        mesh.set_indices(Some(Indices::U32(
                            trimesh
                                .indices()
                                .iter()
                                .flat_map(|triangle| triangle.iter())
                                .cloned()
                                .collect(),
                        )));
                        mesh
                    }
                    _ => unimplemented!(),
                };

                let scale = match shape.shape_type() {
                    #[cfg(feature = "dim2")]
                    ShapeType::Cuboid => {
                        let c = shape.as_cuboid().unwrap();
                        Vec3::new(c.half_extents.x, c.half_extents.y, 1.0)
                    }
                    #[cfg(feature = "dim3")]
                    ShapeType::Cuboid => {
                        let c = shape.as_cuboid().unwrap();
                        Vec3::from_slice_unaligned(c.half_extents.as_slice())
                    }
                    ShapeType::Ball => {
                        let b = shape.as_ball().unwrap();
                        Vec3::new(b.radius, b.radius, b.radius)
                    }
                    ShapeType::Trimesh => Vec3::one(),
                    _ => unimplemented!(),
                } * configuration.scale;

                // NOTE: we can't have both the Scale and NonUniformScale components.
                // However PbrComponents automatically adds a Scale component. So
                // we add each of its field manually except for Scale.
                // That's a bit messy so surely there is a better way?
                let ground_pbr = PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(color.into()),
                    transform: Transform::from_scale(scale),
                    ..Default::default()
                };

                commands.insert(entity, ground_pbr);
            }
        }
    }
}
