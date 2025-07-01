use std::array;
use std::cell::RefCell;
use std::rc::Rc;

use graphics::buffers::VertexArray;

use crate::modelling::test_models::vertex_array_quad;

use super::Builder;
use graphics::buffers::{ActiveFramebuffer, fb_traits::FramebufferWithoutExtra};
use super::QuadVertex;
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::{FlatTexture, Texture};
use graphics::Result;

#[derive(Debug)]
pub struct Quad<const N: usize> {
    pub(crate) vertex_array: VertexArray<QuadVertex>,
    pub texture: [Rc<RefCell<FlatTexture>>; N],
}

impl<const N: usize> Quad<N> {
    pub fn builder() -> Builder<N> {
        Builder::default()
    }

    pub fn new(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        texture: [Rc<RefCell<FlatTexture>>; N],
    ) -> Self {
        Self::builder()
            .vertex_array(vertex_array_quad(left, right, bottom, top))
            .texture(texture)
            .build()
    }

    pub fn screen(texture: [Rc<RefCell<FlatTexture>>; N]) -> Self {
        Self::new(-1.0, 1.0, -1.0, 1.0, texture)
    }

    pub fn downcast<const M: usize>(self) -> Quad<M> {
        let mut iter = self.texture.into_iter();
        let texture = array::from_fn(|_| iter.next().unwrap_or_default());

        Quad {
            vertex_array: self.vertex_array,
            texture,
        }
    }

    pub(crate) fn draw<'a, const OUT: usize, I: FramebufferWithoutExtra<OUT>>(
        &'a self,
        mut active_shader: ActiveShaderProgram<'_, '_, 'a, Self, I::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, I>,
    ) -> Result<()> {
        let input = self.texture
            .iter()
            .map(|tex| /*SAFETY: ref only lives for this function call*/ unsafe { tex.try_borrow_unguarded().expect("the only borrow") as &dyn Texture} ) 
            .enumerate()
            .map(|(index, tex)| (format!("in_texture{}", index), tex));

        active_shader.register_texture(input);

        self.vertex_array.draw(&active_shader, active_framebuffer)
    }
}
