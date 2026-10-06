use alloc::rc::Rc;

use geometry::{Pose, TransformBy as _};
use graphics::shader_program::CullFace;
use utils::{HasIterWithCloned as _, builder, new};

use super::{Skeleton, SkeletonCubic};
use crate::modelling::cubic::BoneAndMesh;
use crate::modelling::cubic::material::Material;
use crate::modelling::cubic::mesh::MeshBuilder;
use crate::modelling::cubic::simple_cubic::TransformType;
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;
use crate::modelling::{SimpleVertex, test_models};

#[derive(Debug, Clone, Default)]
#[must_use = "`SkeletonCubic` must be built with the `build` method"]
pub struct Builder<'a> {
    mesh_builders: Vec<(&'a SkeletonBoneIndex, MeshBuilder<SimpleVertex>)>,
    cull_face: CullFace,
    skeleton: Option<Skeleton>,
    transforms: Vec<TransformType>,
}

impl<'a> Builder<'a> {
    new!();

    builder!(cull_face: CullFace);

    builder!(skeleton: Option<Skeleton>);

    pub fn push_mesh(&mut self, bone: &'a SkeletonBoneIndex, mesh: MeshBuilder<SimpleVertex>) {
        self.mesh_builders.push((bone, mesh));
    }

    pub fn extend_meshes<I>(&mut self, meshes: I)
    where
        I: IntoIterator<Item = (&'a SkeletonBoneIndex, MeshBuilder<SimpleVertex>)>,
    {
        self.mesh_builders.extend(meshes);
    }

    pub fn push_cube(
        &mut self,
        pose: Pose,
        material: Rc<Material>,
        side_length: f32,
        bone: &'a SkeletonBoneIndex,
    ) {
        let mut mesh_builder = MeshBuilder::new().material(material);
        mesh_builder.extend_triangle(
            test_models::origin_cube(side_length).map(|triangle| triangle.transform_by(pose)),
        );
        self.mesh_builders.push((bone, mesh_builder));
    }

    pub fn material(mut self, material: Rc<Material>) -> Self {
        for ((_, mesh), material) in self.mesh_builders.iter_mut().with_cloned(material) {
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

    pub fn build(self) -> SkeletonCubic {
        let transforms = self.transforms.into();

        let meshes = self
            .mesh_builders
            .into_iter()
            .map(|(bone, builder)| BoneAndMesh {
                bone: bone.clone_inner(),
                mesh: builder.transforms(Rc::clone(&transforms)).build(),
            })
            .collect::<Vec<_>>();

        SkeletonCubic {
            meshes,
            cull_face: self.cull_face,
            skeleton: self.skeleton.unwrap_or_default(),
        }
    }
}
