use std::rc::Rc;

use graphics::vertex_array::VertexArray;
use utils::{builder, new};

use crate::modelling::advanced_vertex::IncompleteVertex;
use crate::modelling::cubic::Mesh;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::cubic::material::Material;
use crate::modelling::{CubicVertex, ExtendedVertex, SimpleVertex};

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct Builder {
    pub triangles: Vec<[IncompleteVertex<3>; 3]>,
    pub material: Rc<Material>,
    pub scale: Option<f32>,
    transform: Option<Pose>,
}


impl Builder {
    new!();

    builder!(scale: Option<f32>);

    builder!(material: Rc<Material>);

    builder!(transform, opt_transform: Option<Pose>);

    pub fn push_triangle(&mut self, triangle: [IncompleteVertex<3>; 3]) {
        self.triangles.push(triangle);
    }

    pub fn extend_triangle<I: IntoIterator<Item = [IncompleteVertex<3>; 3]>>(
        &mut self,
        triangles: I,
    ) {
        self.triangles.extend(triangles);
    }

    pub fn extend_complete_triangle<I: IntoIterator<Item = [SimpleVertex; 3]>>(
        &mut self,
        triangles: I,
    ) {
        self.extend_triangle(triangles.into_iter().map(|arr| {
            arr.map(
                |SimpleVertex {
                     position,
                     texture,
                     normal,
                     tangent,
                 }| IncompleteVertex {
                    position,
                    texture: Some(texture),
                    normal: Some(normal),
                    tangent: Some(tangent),
                    ..IncompleteVertex::<3>::default()
                },
            )
        }));
    }

    pub fn build<V: CubicVertex>(mut self) -> Mesh<V>
    where
        [(); V::ELEMENT_COUNT]:,
    {
        if let Some(scale) = self.scale {
            for triangle in &mut self.triangles {
                for vertex in triangle {
                    vertex.position *= scale;
                }
            }
        }
        // if let Some(transform) = self.transformation {
        // for triangle in &mut self.triangles {
        // triangle = triangle.transform_by(&transform);
        // }
        // }

        let mut vertex_array_builder = VertexArray::builder();
        vertex_array_builder.extend_triangles(
            self.triangles
                .into_iter()
                .filter_map(ExtendedVertex::from_incomplete_triangle),
        );

        let vertex_array = vertex_array_builder.build();

        Mesh::new(Rc::new(vertex_array), self.material)
    }
}
