use std::array;
use std::cell::RefCell;
use std::rc::Rc;

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
pub enum DrawError<'a> {
    VertexArrayDraw(VertexArrayDrawError<'a>),
}
error_boilerplate!(<'a> DrawError<'a>);

impl<const N: usize> Quad<N> {
    pub(crate) fn draw<'a, 'b, const OUT: usize, O, I: FramebufferWithoutExtra<OUT>>(
        &'b self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'b, Self, O, I::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, I>,
    ) -> Result<(), DrawError<'a>> {
        _ = active_shader.cull_face(CullFace::BackFace);

        let input = self.texture
            .iter()
            //.map(|tex| /*SAFETY: ref only lives for this function call*/ unsafe { tex.try_borrow_unguarded().expect("the only borrow") as &dyn Texture} ) 
            .map(|tex| &**tex as &dyn Texture )
            .enumerate()
            .map(|(index, tex)| (format!("in_texture{index}"), tex));

        active_shader.register_texture(input);

        self.vertex_array
            .draw(active_shader, active_framebuffer)
            .map_err(|err| DrawError::VertexArrayDraw(err))?;

        Ok(())
    }
}

impl<const N: usize> Model for Quad<N> {
    type Vertex = QuadVertex;
}
