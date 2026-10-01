use alloc::rc::Rc;

use geometry::{Pose, TransformBy as _};
use graphics::shader_program::CullFace;
use utils::{HasIterWithCloned as _, builder, new};

use super::SimpleCubic;
use crate::modelling::cubic::material::Material;
use crate::modelling::cubic::mesh::MeshBuilder;
use crate::modelling::{SimpleVertex, test_models};

#[derive(Debug, Clone, Default)]
#[must_use = "SimpleCubic must by built with the `build method"]
pub struct Builder {
    mesh_builders: Vec<MeshBuilder<SimpleVertex>>,
    transforms: Vec<TransformType>,
    // transform: Option<Pose>,
    cull_face: CullFace,
    // scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum TransformType {
    ByPose(Pose),
    Scale(f32),
}

impl Builder {
    new!();

    builder!(cull_face: CullFace);

    pub fn push_mesh(&mut self, mesh: MeshBuilder<SimpleVertex>) {
        self.mesh_builders.push(mesh);
    }

    pub fn extend_meshes<I: IntoIterator<Item = MeshBuilder<SimpleVertex>>>(&mut self, meshes: I) {
        self.mesh_builders.extend(meshes);
    }

    pub fn push_cube(&mut self, pose: Pose, material: Rc<Material>, side_length: f32) {
        let mut mesh_builder = MeshBuilder::new().material(material);
        mesh_builder.extend_triangle(
            test_models::origin_cube(side_length).map(|triangle| triangle.transform_by(pose)),
        );
        self.mesh_builders.push(mesh_builder);
    }

    pub fn material(mut self, material: Rc<Material>) -> Self {
        for (mesh, material) in self.mesh_builders.iter_mut().with_cloned(material) {
            mesh.material = material;
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
