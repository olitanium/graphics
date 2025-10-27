use std::path::Path;
use std::rc::Rc;

use graphics::ApiResult;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use russimp::scene::PostProcess;

use super::material::Material;
use super::{Builder, Mesh, Skeleton, import};
use crate::error::EngResult;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::simple_vertex::SimpleVertex;

#[derive(Clone, Debug)]
pub struct Cubic {
    pub(crate) meshes: Vec<(usize, Mesh<SimpleVertex>)>,
    pub cull_face: CullFace,
    pub skeleton: Skeleton,
    // pub relative: bool,
}

impl Default for Cubic {
    fn default() -> Self {
        // Self::cube(1.0, Default::default()).build()
        Self::empty()
    }
}

impl Cubic {
    pub fn empty() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn cube(side_length: f32, material: Rc<Material>) -> Builder {
        let mut out = Self::builder();
        out.push_cube(material, side_length, 0);
        out
    }

    pub fn temp_set_all_material(&mut self, mat: Rc<Material>) {
        self.meshes
            .iter_mut()
            .for_each(|(_, mesh)| mesh.material = mat.clone());
    }

    pub fn triangles(&self) -> Vec<(usize, [SimpleVertex; 3])> {
        self.meshes
            .iter()
            .map(|&(bone, ref mesh)| {
                mesh.triangles()
                    .into_owned()
                    .into_iter()
                    .map(move |triangles| (bone, triangles))
            })
            .flatten()
            .collect()
    }

    pub fn root(&self) -> Pose {
        self.skeleton.root
    }

    pub fn root_mut(&mut self) -> &mut Pose {
        &mut self.skeleton.root
    }

    pub(crate) fn draw<'a, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, SimpleVertex, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        animation: usize,
        time: f32,
    ) -> EngResult<(), !> {
        // Ignore cull_face error
        let _: ApiResult<_, !> = active_shader.cull_face(self.cull_face);
        for &(bone, ref mesh) in self.meshes.iter() {
            active_shader.set_uniform(
                "model",
                &self
                    .skeleton
                    .get_bone_pose(false, bone, animation, time)
                    .as_matrix(),
            );

            mesh.draw(active_shader, active_framebuffer)?;
        }

        Ok(())
    }

    pub fn import<NEVER: Clone, PA: AsRef<Path>>(
        path: PA,
        post_process: Vec<PostProcess>,
        load_tangent_space: bool,
    ) -> EngResult<Builder, NEVER> {
        import::import(path, post_process, load_tangent_space)
    }
}
