use std::borrow::Cow;
use std::path::Path;
use std::rc::Rc;

use graphics::ApiResult;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use graphics::vertex_array::VertexArray;
use russimp::scene::PostProcess;

use super::geometry::YieldsPose;
use super::material::Material;
use super::{Builder, Skeleton, import};
use crate::error::EngResult;
use super::Mesh;
use crate::modelling::simple_vertex::SimpleVertex;
use crate::modelling::test_models;

#[derive(Clone, Debug)]
pub struct Cubic {
    pub(crate) meshes: Vec<Mesh>,
    pub cull_face: CullFace,
    pub skeleton: Skeleton,
    pub relative: bool,
    //pub scale: f32,
}

impl Default for Cubic {
    fn default() -> Self {
        Self::cube(1.0, Default::default()).build()
    }
}

impl Cubic {
    pub fn empty() -> Self {
        Self::builder()
            //.push_mesh_from(Rc::new(VertexArray::empty()), Rc::new(Material::blank()), 0)
            .build()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn cube(side_length: f32, material: Rc<Material>) -> Builder {
        //let vertex_array = Rc::new(test_models::origin_cube(side_length));
        let mut out = Self::builder();
        out.push_cube(material, side_length, 0);
        out
    }

    pub fn temp_set_all_material(&mut self, mat: Rc<Material>) {
        self.meshes
            .iter_mut()
            .for_each(|mesh| mesh.material = mat.clone());
    }

    pub fn triangles(&self) -> Vec<[SimpleVertex; 3]> {
        self.meshes
            .iter()
            .map(|mesh| mesh.triangles().into_owned())
            .flatten()
            .collect()
    }

    pub(crate) fn draw<'a, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        animation: usize,
        time: f32,
    ) -> EngResult<(), !> {
        // Ignore cull_face error
        let _: ApiResult<_, !> = active_shader.cull_face(self.cull_face);

        for mesh in &self.meshes {
            mesh.draw(
                active_shader,
                active_framebuffer,
                &self.skeleton,
                self.relative,
                animation,
                time,
                //self.scale,
                // TODO: remove
                1.0,
            )?;
        }

        Ok(())
    }

    pub fn import<NEVER: Clone, PA: AsRef<Path>>(
        path: PA,
        post_process: Vec<PostProcess>,
    ) -> EngResult<Builder, NEVER> {
        import::import(path, post_process)
    }
}
