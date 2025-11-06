use std::rc::Rc;

use graphics::shader_program::CullFace;
use utils::{builder, new};

use super::{SimpleCubic, Skeleton};
use crate::TransformBy;
use crate::modelling::cubic::BoneAndMesh;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::cubic::material::Material;
use crate::modelling::cubic::mesh::MeshBuilder;
use crate::modelling::test_models;

#[derive(Debug, Clone)]
pub struct Builder {
    mesh_builders: Vec<(usize, MeshBuilder)>,
    cull_face: CullFace,
    skeleton: Option<Skeleton>,
    scale: f32,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            mesh_builders: Default::default(),
            cull_face: Default::default(),
            skeleton: Default::default(),
            scale: 1.0,
        }
    }
}

impl Builder {
    new!();

    builder!(cull_face: CullFace);

    builder!(skeleton: Option<Skeleton>);

    builder!(scale: f32);

    pub fn push_mesh(&mut self, bone: usize, mesh: MeshBuilder) {
        self.mesh_builders.push((bone, mesh));
    }

    pub fn extend_meshes<I: IntoIterator<Item = (usize, MeshBuilder)>>(&mut self, meshes: I) {
        self.mesh_builders.extend(meshes);
    }

    pub fn push_cube(&mut self, pose: Pose, material: Rc<Material>, side_length: f32, bone: usize) {
        let mut mesh_builder = MeshBuilder::new().material(material);
        mesh_builder.extend_triangle(
            test_models::origin_cube(side_length).map(|triangle| triangle.transform_by(&pose)),
        );
        self.mesh_builders.push((bone, mesh_builder));
    }

    pub fn material(mut self, material: Rc<Material>) -> Self {
        for (_, mesh) in &mut self.mesh_builders {
            mesh.material = material.clone();
        }
        self
    }

    pub fn build(self) -> SimpleCubic {
        let meshes = self
            .mesh_builders
            .into_iter()
            .map(|(bone, builder)| BoneAndMesh {
                bone,
                mesh: builder.scale(self.scale).build(),
            })
            .collect::<Vec<_>>();

        SimpleCubic {
            meshes,
            cull_face: self.cull_face,
            skeleton: self.skeleton.unwrap_or_default(),
        }
    }
}
