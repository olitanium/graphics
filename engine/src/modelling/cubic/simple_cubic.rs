use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;

use graphics::ApiResult;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use russimp::scene::PostProcess;
use utils::partition;

use super::material::Material;
use super::{Mesh, Skeleton, import};
use crate::entity::EntityInner;
use crate::error::EngResult;
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose, SkeletalAnimation};
use crate::modelling::simple_vertex::SimpleVertex;
use crate::modelling::{SkeletonBb, test_models};

mod group;
pub use group::Group;

mod builder;
pub use builder::Builder;

#[derive(Clone, Debug, Default)]
pub struct SimpleCubic {
    pub(crate) meshes: Vec<BoneAndMesh>,
    pub cull_face: CullFace,
    pub skeleton: Skeleton,
}

#[derive(Debug, Clone)]
pub struct BoneAndMesh {
    pub(crate) bone: usize,
    pub(crate) mesh: Mesh<SimpleVertex>,
}

impl SimpleCubic {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn cube(side_length: f32, material: Rc<Material>) -> Builder {
        let mut out = Self::builder();
        out.push_cube(Pose::IDENTITY, material, side_length, 0);
        out
    }

    pub fn tail(max_len: usize) -> (Self, SkeletalAnimation) {
        test_models::tail_simplecubic(max_len)
    }

    pub fn temp_set_all_material(&mut self, mat: Rc<Material>) {
        self.meshes
            .iter_mut()
            .for_each(|BoneAndMesh { mesh, .. }| mesh.material = mat.clone());
    }

    pub fn triangles(&self) -> Vec<(usize, [SimpleVertex; 3])> {
        self.meshes
            .iter()
            .flat_map(|BoneAndMesh { bone, mesh }| {
                mesh.triangles()
                    .into_owned()
                    .into_iter()
                    .map(move |triangles| (*bone, triangles))
            })
            .collect()
    }

    pub fn import<NEVER: Clone, PA: AsRef<Path>>(
        path: PA,
        post_process: Vec<PostProcess>,
        load_tangent_space: bool,
    ) -> EngResult<Builder, NEVER> {
        let import_output = import::import(path, post_process, load_tangent_space)?;

        let mut out = Self::builder();
        out.extend_meshes(import_output.meshes.into_iter().map(|builder| (0, builder)));

        Ok(out)
    }

    pub(crate) fn draw<'a, NEVER, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, SimpleVertex, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> EngResult<(), NEVER> {
        // Ignore cull_face error
        let _: ApiResult<_, !> = active_shader.cull_face(self.cull_face);
        for BoneAndMesh { bone, mesh } in self.meshes.iter() {
            active_shader.set_uniform(
                "model",
                &calculated_animation
                    .get(*bone)
                    .map_or_else(|_| Matrix::identity(), Pose::as_transformation_matrix),
            )?;

            mesh.draw(active_shader, active_framebuffer)?;
        }

        Ok(())
    }
}

impl EntityInner for SimpleCubic {
    type BoundingBox<T: Debug> = SkeletonBb<SimpleVertex, T>;
    type Vertex = SimpleVertex;

    fn make_bounding_box<
        T: Debug,
        F: FnMut(&[(graphics::linear_algebra::Vector<3>, [Self::Vertex; 3])]) -> T,
    >(
        &self,
        depth: usize,
        init: F,
    ) -> Option<Self::BoundingBox<T>> {
        let per_mesh = partition(self.triangles(), |(bone, _)| vec![*bone], |(_, tri)| tri);

        SkeletonBb::from_vec_triangle(per_mesh, depth, self.skeleton.clone(), init)
    }

    fn skeleton(&self) -> &Skeleton {
        &self.skeleton
    }

    fn draw<'a, NEVER, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self::Vertex, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> EngResult<(), NEVER> {
        self.draw(active_shader, active_framebuffer, calculated_animation)
    }
}
