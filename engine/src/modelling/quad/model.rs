use std::cell::RefCell;
use std::rc::Rc;
use std::{array, mem};

use graphics::ApiResult;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use graphics::texture::{FlatTexture, Texture};
use graphics::vertex_array::VertexArray;
use utils::RefOwned;

use super::{Builder, QuadVertex};
use crate::EngResult;
use crate::modelling::test_models;

#[derive(Debug)]
pub struct Quad<const N: usize> {
    pub(crate) vertex_array: RefOwned<'static, VertexArray<QuadVertex>>,
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
            .vertex_array(test_models::vertex_array_quad(left, right, bottom, top))
            .texture(texture)
            .build()
    }

    pub fn screen(texture: [Rc<RefCell<FlatTexture>>; N]) -> Self {
        Quad {
            vertex_array: RefOwned::Ref(test_models::screen_quad()),
            texture,
        }
    }

    pub fn downcast<const M: usize>(mut self) -> Quad<M> {
        let mut iter = self.texture.iter_mut().map(|x| mem::take(x));
        let texture = array::from_fn(|_| iter.next().unwrap_or_default());
        Quad {
            vertex_array: mem::take(&mut self.vertex_array),
            texture,
        }
    }

    pub(crate) fn draw<'a, const OUT: usize, I: FramebufferWithoutExtra<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, I::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, I>,
    ) -> EngResult<(), !> {
        let _: ApiResult<_, !> = active_shader.cull_face(CullFace::BackFace);

        let input = self.texture
            .iter()
            .map(|tex| /*SAFETY: ref only lives for this function call*/ unsafe { tex.try_borrow_unguarded().expect("the only borrow") as &dyn Texture} ) 
            .enumerate()
            .map(|(index, tex)| (format!("in_texture{index}"), tex));

        active_shader.register_texture(input);

        self.vertex_array.draw(active_shader, active_framebuffer)?;

        Ok(())
    }
}
