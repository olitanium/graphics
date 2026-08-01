use std::fmt::Debug;
use std::rc::Rc;

use graphics::Model;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::{ActiveShaderProgram, CullFace, ShaderSetUniformError};
use utils::{error_boilerplate, partition};

use super::material::Material;
use super::{Mesh, Skeleton};
use crate::entity::EntityInner;
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose, SkeletalAnimation};
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;
use crate::modelling::cubic::{SimpleCubic, mesh};
use crate::modelling::{SimpleVertex, SkeletonBb, test_models};

mod group;
pub use group::Group;

mod builder;
pub use builder::Builder;

#[derive(Clone, Debug, Default)]
pub struct SkeletonCubic {
    pub(crate) meshes: Vec<BoneAndMesh>,
    pub cull_face: CullFace,
    pub skeleton: Skeleton,
}

#[derive(Debug)]
pub struct BoneAndMesh {
    pub(crate) bone: SkeletonBoneIndex,
    pub(crate) mesh: Mesh<SimpleVertex>,
}

impl Clone for BoneAndMesh {
    fn clone(&self) -> Self {
        Self {
            bone: self.bone.clone_inner(),
            mesh: self.mesh.clone(),
        }
    }
}

impl SkeletonCubic {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn builder<'a>() -> Builder<'a> {
        Builder::new()
    }

    pub fn cube(side_length: f32, material: Rc<Material>) -> Builder<'static> {
        let mut out = Self::builder();
        out.push_cube(
            Pose::IDENTITY,
            material,
            side_length,
            &SkeletonBoneIndex::SKELETON_ROOT_BONE,
        );
        out
    }

    pub fn tail(max_len: usize) -> (Self, SkeletalAnimation) {
        let (a, _, b) = test_models::tail_simplecubic(max_len);
        (a, b)
    }

    pub fn temp_set_all_material(&mut self, mat: Rc<Material>) {
        self.meshes
            .iter_mut()
            .for_each(|BoneAndMesh { mesh, .. }| mesh.material = mat.clone());
    }

    pub fn triangles(&self) -> Vec<(&SkeletonBoneIndex, [SimpleVertex; 3])> {
        self.meshes
            .iter()
            .flat_map(|BoneAndMesh { bone, mesh }| {
                mesh.triangles()
                    .into_owned()
                    .into_iter()
                    .map(move |triangles| (bone, triangles))
            })
            .collect()
    }

    // pub fn import<P: AsRef<Path>>(
    // path: P,
    // post_process: Vec<PostProcess>,
    // load_tangent_space: bool,
    // ) -> Result<Builder, import::ImportError> {
    // let import_output = import::import(path, post_process, load_tangent_space)?;
    //
    // let mut out = Self::builder();
    // out.extend_meshes(
    // import_output
    // .meshes
    // .into_iter()
    // .map(|builder| (SkeletonBoneIndex::SKELETON_ROOT_BONE, builder)),
    // );
    //
    // Ok(out)
    // }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError<'a> {
    Mesh(mesh::DrawError<'a>),
    SetUniform(ShaderSetUniformError<'a>),
}
error_boilerplate!(<'a> DrawError<'a>);

impl SkeletonCubic {
    pub(crate) fn draw<'a, 'c, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'c self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'c, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), DrawError<'a>> {
        // Ignore cull_face error
        _ = active_shader.cull_face(self.cull_face);
        for BoneAndMesh { bone, mesh } in self.meshes.iter() {
            active_shader
                .set_uniform(
                    "model",
                    &calculated_animation
                        .get(bone, &self.skeleton)
                        .map_or_else(|_| Matrix::identity(), Pose::as_transformation_matrix),
                )
                .map_err(DrawError::SetUniform)?;

            mesh.draw(active_shader, active_framebuffer)
                .map_err(DrawError::Mesh)?;
        }

        Ok(())
    }
}

impl EntityInner for SkeletonCubic {
    type BoundingBox<T> = SkeletonBb<<SkeletonCubic as Model>::Vertex, T>;
    type DrawError<'a> = DrawError<'a>;

    fn make_bounding_box<
        T,
        F: FnMut(&[(graphics::linear_algebra::Vector<3>, [Self::Vertex; 3])]) -> T,
    >(
        &self,
        depth: usize,
        init: F,
    ) -> Option<Self::BoundingBox<T>> {
        let per_mesh = partition(&self.triangles(), |(bone, _)| vec![bone.0], |(_, tri)| tri);

        SkeletonBb::from_vec_triangle(per_mesh, depth, init)
    }

    fn skeleton(&self) -> &Skeleton {
        &self.skeleton
    }

    fn draw<'a, 'b, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'b, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), Self::DrawError<'b>> {
        self.draw(active_shader, active_framebuffer, calculated_animation)
    }
}

impl Model for SkeletonCubic {
    type Vertex = SimpleVertex;
}

impl From<SimpleCubic> for SkeletonCubic {
    fn from(value: SimpleCubic) -> Self {
        let SimpleCubic { meshes, cull_face } = value;

        let meshes = meshes
            .into_iter()
            .map(|mesh| BoneAndMesh {
                bone: SkeletonBoneIndex::SKELETON_ROOT_BONE,
                mesh,
            })
            .collect();
        let skeleton = Skeleton::builder().build();
        Self {
            meshes,
            cull_face,
            skeleton,
        }
    }
}
