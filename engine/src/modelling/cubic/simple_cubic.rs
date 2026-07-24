use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;

use graphics::Model;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::{ActiveShaderProgram, CullFace, ShaderSetUniformError};
use russimp::scene::PostProcess;
use utils::error_boilerplate;

use super::material::Material;
use super::{Mesh, Skeleton, import};
use crate::entity::EntityInner;
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose};
use crate::modelling::cubic::mesh;
use crate::modelling::{BoundingBox, SimpleVertex};

mod group;
pub use group::Group;

mod builder;
pub use builder::Builder;
pub(crate) use builder::TransformType;

#[derive(Clone, Debug, Default)]
pub struct SimpleCubic {
    pub(crate) meshes: Vec<Mesh<SimpleVertex>>,
    pub cull_face: CullFace,
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
        out.push_cube(Pose::IDENTITY, material, side_length);
        out
    }

    pub fn temp_set_all_material(&mut self, mat: Rc<Material>) {
        self.meshes
            .iter_mut()
            .for_each(|mesh| mesh.material = mat.clone());
    }

    pub fn triangles(&self) -> Vec<[<SimpleCubic as Model>::Vertex; 3]> {
        self.meshes
            .iter()
            .flat_map(|mesh| mesh.triangles().into_owned())
            .collect()
    }

    pub fn import<P: AsRef<Path>>(
        path: P,
        post_process: Vec<PostProcess>,
        load_tangent_space: bool,
    ) -> Result<Builder, import::ImportError> {
        let import_output = import::import(path, post_process, load_tangent_space)?;

        let mut out = Self::builder();
        out.extend_meshes(import_output.meshes.into_iter());

        Ok(out)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError<'a> {
    Mesh(mesh::DrawError<'a>),
    SetUniform(ShaderSetUniformError<'a>),
}
error_boilerplate!(<'a> DrawError<'a>);

impl SimpleCubic {
    pub(crate) fn draw<'a, 'c, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'c self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'c, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        pose: Pose,
    ) -> Result<(), DrawError<'a>> {
        // Ignore cull_face error
        _ = active_shader.cull_face(self.cull_face);
        active_shader
            .set_uniform("model", &pose.as_transformation_matrix())
            .map_err(|err| DrawError::SetUniform(err))?;

        for mesh in self.meshes.iter() {
            mesh.draw(active_shader, active_framebuffer)
                .map_err(|err| DrawError::Mesh(err))?;
        }

        Ok(())
    }
}

impl EntityInner for SimpleCubic {
    type BoundingBox<T> = BoundingBox<<SimpleCubic as Model>::Vertex, 3, T>;
    type DrawError<'a> = DrawError<'a>;

    fn make_bounding_box<
        T,
        F: FnMut(&[(graphics::linear_algebra::Vector<3>, [Self::Vertex; 3])]) -> T,
    >(
        &self,
        depth: usize,
        init: F,
    ) -> Option<Self::BoundingBox<T>> {
        let triangles = self.triangles();
        Self::BoundingBox::new_from_triangles(triangles, depth, init)
    }

    fn skeleton(&self) -> &Skeleton {
        Skeleton::just_root()
    }

    fn draw<'a, 'b, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'b, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), Self::DrawError<'b>> {
        self.draw(
            active_shader,
            active_framebuffer,
            calculated_animation.root(),
        )
    }
}

impl Model for SimpleCubic {
    type Vertex = SimpleVertex;
}
