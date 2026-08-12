use std::rc::Rc;

use geometry::{Pose, TransformBy};
use graphics::shader_program::CullFace;
use utils::{builder, new};

use super::SimpleCubic;
use crate::modelling::cubic::material::Material;
use crate::modelling::cubic::mesh::MeshBuilder;
use crate::modelling::test_models;

#[derive(Debug, Clone, Default)]
pub struct Builder {
    mesh_builders: Vec<MeshBuilder>,
    transforms: Vec<TransformType>,
    // transform: Option<Pose>,
    cull_face: CullFace,
    // scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TransformType {
    ByPose(Pose),
    Scale(f32),
}

impl Builder {
    new!();

    builder!(cull_face: CullFace);

    pub fn push_mesh(&mut self, mesh: MeshBuilder) {
        self.mesh_builders.push(mesh);
    }

    pub fn extend_meshes<I: IntoIterator<Item = MeshBuilder>>(&mut self, meshes: I) {
        self.mesh_builders.extend(meshes);
    }

    pub fn push_cube(&mut self, pose: Pose, material: Rc<Material>, side_length: f32) {
        let mut mesh_builder = MeshBuilder::new().material(material);
        mesh_builder.extend_triangle(
            test_models::origin_cube(side_length).map(|triangle| triangle.transform_by(&pose)),
        );
        self.mesh_builders.push(mesh_builder);
    }

    pub fn material(mut self, material: Rc<Material>) -> Self {
        for mesh in &mut self.mesh_builders {
            mesh.material = material.clone();
        }
        self
    }

    pub fn apply_transformation(mut self, transform: Pose) -> Self {
        self.transforms.push(TransformType::ByPose(transform));
        self
    }

    pub fn apply_inverse_transform(self, transform: Pose) -> Self {
        self.apply_transformation(transform.inverse())
    }

    pub fn scale(mut self, factor: f32) -> Self {
        self.transforms.push(TransformType::Scale(factor));
        self
    }

    pub fn build(self) -> SimpleCubic {
        let transforms = self.transforms.into();

        let meshes = self
            .mesh_builders
            .into_iter()
            .map(move |builder| builder.transforms(Rc::clone(&transforms)).build())
            .collect::<Vec<_>>();

        SimpleCubic {
            meshes,
            cull_face: self.cull_face,
        }
    }
}
