use std::path::Path;
use std::rc::Rc;

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use graphics::vertex_array::VertexArray;
use russimp::scene::PostProcess;

use super::geometry::YieldsPose;
use super::material::Material;
use super::{Builder, Skeleton, import};
use crate::error::Result;
use crate::modelling::simple_vertex::SimpleVertex;
use crate::modelling::test_models::vertex_array_cube;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertex_array: Rc<VertexArray<SimpleVertex>>,
    pub material: Rc<Material>,
    pub bone: usize,
}

impl Mesh {
    pub fn new(
        vertex_array: Rc<VertexArray<SimpleVertex>>,
        material: Rc<Material>,
        bone: usize,
    ) -> Self {
        Self {
            vertex_array,
            material,
            bone,
        }
    }

    pub(crate) fn draw<'a, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, (Cubic, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        skeleton: &Skeleton,
        relative: bool,
        animation: usize,
        time: f32,
        scale: f32,
    ) -> graphics::Result<()> {
        self.material.register_to(active_shader, "material");

        active_shader.set_uniform(
            "model".into(),
            skeleton
                .get_pose((relative, self.bone, animation, time))
                .as_matrix()
                * Matrix::transform_scale(scale, scale, scale),
        );

        self.vertex_array.draw(active_shader, active_framebuffer)
    }
}

#[derive(Clone, Debug)]
pub struct Cubic {
    pub(crate) meshes: Vec<Mesh>,
    pub cull_face: CullFace,
    // pub animation: Animation,
    pub skeleton: Skeleton,
    pub realtive: bool,
    pub scale: f32,
}

impl Default for Cubic {
    fn default() -> Self {
        Self::cube(1.0, Rc::new(Material::blank())).build()
    }
}

impl Cubic {
    pub fn empty() -> Self {
        Self::builder()
            .push_mesh_from(Rc::new(VertexArray::empty()), Rc::new(Material::blank()), 0)
            .build()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn cube(side_length: f32, material: Rc<Material>) -> Builder {
        let vertex_array = Rc::new(vertex_array_cube(side_length));
        Self::builder().push_mesh_from(vertex_array, material, 0)
    }

    pub fn temp_set_all_material(&mut self, mat: Rc<Material>) {
        self.meshes
            .iter_mut()
            .for_each(|mesh| mesh.material = mat.clone());
    }

    pub(crate) fn draw<'a, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        animation: usize,
        time: f32,
    ) -> graphics::Result<()> {
        // Ignore cull_face error
        _ = active_shader.cull_face(self.cull_face);

        // active_shader.set_uniform("model".to_string(), self.model_matrix(hint));

        for mesh in &self.meshes {
            mesh.draw(
                active_shader,
                active_framebuffer,
                &self.skeleton,
                self.realtive,
                animation,
                time,
                self.scale,
            )?;
        }

        Ok(())
    }

    pub fn import<PA: AsRef<Path>>(path: PA, post_process: Vec<PostProcess>) -> Result<Builder> {
        import::import(path, post_process)
    }
}
