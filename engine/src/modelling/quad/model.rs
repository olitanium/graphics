use alloc::ffi::CString;
use alloc::rc::Rc;
use core::array;

use graphics::Model;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::linear_algebra::Vector;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use graphics::texture::{FlatTexture, Texture};
use graphics::vertex_array::{VertexArray, VertexArrayDrawError};
use utils::{RefOwned, error_boilerplate};

use super::{Builder, QuadVertex};
use crate::modelling::test_models;

#[derive(Debug)]
pub struct Quad<const N: usize> {
    pub(crate) vertex_array: RefOwned<'static, VertexArray<QuadVertex>>,
    pub texture: [Rc<FlatTexture>; N],
}

impl<const N: usize> Quad<N> {
    pub fn builder() -> Builder<N> {
        Builder::default()
    }

    pub fn new(
        lower_bound: Vector<2>,
        upper_bound: Vector<2>,
        texture: [Rc<FlatTexture>; N],
    ) -> Self {
        Self::builder()
            .vertex_array(test_models::vertex_array_quad(lower_bound, upper_bound))
            .texture(texture)
            .build()
    }

    // pub fn screen(texture: [Rc<FlatTexture>; N]) -> Self {
    // Quad {
    // vertex_array: RefOwned::Ref(test_models::screen_quad()),
    // texture,
    // }
    // }

    pub fn downcast<const M: usize>(self) -> Quad<M> {
        let Self {
            vertex_array,
            texture,
        } = self;

        let mut iter = texture.into_iter();
        let texture = array::from_fn(|_| iter.next().unwrap_or_default());
        Quad {
            vertex_array,
            texture,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError {
    VertexArrayDraw(VertexArrayDrawError),
}
error_boilerplate!(DrawError);

impl<const N: usize> Quad<N> {
    pub(crate) fn draw<'b, const OUT: usize, O, I>(
        &'b self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'b, Self, O, I::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, I>,
    ) -> Result<(), DrawError>
    where
        I: FramebufferWithoutExtra<OUT>,
    {
        _ = active_shader.cull_face(CullFace::BackFace);

        let input = self
            .texture
            .iter()
            .map(|tex| &**tex as &dyn Texture)
            .enumerate()
            .filter_map(|(index, tex)| {
                CString::new(format!("in_texture{index}"))
                    .map(|cstr| (cstr, tex))
                    .ok()
            });

        active_shader.register_texture(input);

        self.vertex_array
            .draw(active_shader, active_framebuffer)
            .map_err(DrawError::VertexArrayDraw)?;

        Ok(())
    }
}

impl<const N: usize> Model for Quad<N> {
    type Vertex = QuadVertex;
}
