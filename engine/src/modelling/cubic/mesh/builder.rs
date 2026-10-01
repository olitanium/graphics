use alloc::rc::Rc;
use core::marker::PhantomData;

use geometry::TransformBy as _;
use graphics::vertex_array::VertexArray;
use utils::{builder, new};

use crate::modelling::cubic::Mesh;
use crate::modelling::cubic::material::Material;
use crate::modelling::cubic::simple_cubic::TransformType;
use crate::modelling::extended_vertex::IncompleteVertex;
use crate::modelling::{CubicVertex, ExtendedVertex, SimpleVertex};

#[derive(Debug, Clone)]
pub struct Builder<V: CubicVertex> {
    pub triangles: Vec<[IncompleteVertex<3>; 3]>,
    pub material: Rc<Material>,
    transforms: Rc<[TransformType]>,
    phantom: PhantomData<Vec<V>>,
}

impl<V: CubicVertex> Default for Builder<V> {
    fn default() -> Self {
        Self {
            triangles: Vec::default(),
            material: Rc::default(),
            transforms: Rc::default(),
            phantom: PhantomData,
        }
    }
}

impl<V: CubicVertex> Builder<V> {
    new!();

    builder!(material: Rc<Material>);

    builder!(transforms: Rc<[TransformType]>);

    pub fn push_triangle(&mut self, triangle: [IncompleteVertex<3>; 3]) {
        self.triangles.push(triangle);
    }

    pub fn extend_triangle<I>(&mut self, triangles: I) -> &mut Self
    where
        I: IntoIterator<Item = [IncompleteVertex<3>; 3]>,
    {
        self.triangles.extend(triangles);
        self
    }

    pub fn extend_complete_triangle<I>(&mut self, triangles: I)
    where
        I: IntoIterator<Item = [SimpleVertex; 3]>,
    {
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

    pub fn build(mut self) -> Mesh<V> {
        for transform in self.transforms.iter() {
            match transform {
                TransformType::ByPose(pose) => {
                    for triangle in &mut self.triangles {
                        *triangle = triangle.clone().transform_by(*pose);
                    }
                }
                TransformType::Scale(factor) => {
                    for triangle in &mut self.triangles {
                        for vertex in triangle {
                            vertex.position *= *factor;
                        }
                    }
                }
            }
        }

        let mut vertex_array_builder = VertexArray::builder();
        vertex_array_builder.extend_triangles(
            self.triangles
                .into_iter()
                .map(ExtendedVertex::from_incomplete_triangle)
                .inspect(|opt_triangle| {
                    if opt_triangle.is_none() {
                        eprintln!("bad triangle");
                    }
                })
                .flatten(),
        );

        let vertex_array = vertex_array_builder.build();

        Mesh::new(Rc::new(vertex_array), self.material)
    }
}
