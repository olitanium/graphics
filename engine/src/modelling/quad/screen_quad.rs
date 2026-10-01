mod group;

use alloc::ffi::CString;

use graphics::ActiveShaderProgram;
use graphics::framebuffer::traits::{AttachmentWithoutExtra, FramebufferWithoutExtra};
use graphics::framebuffer::{ActiveFramebuffer, Framebuffer};
use graphics::shader_program::CullFace;
use graphics::texture::{FlatTexture, Texture};
use graphics::vertex_array::{VertexArray, VertexArrayDrawError};
pub use group::Group as ScreenQuadGroup;
use utils::error_boilerplate;

use crate::modelling::test_models::screen_quad;
use crate::modelling::{Quad, QuadVertex};

#[derive(Debug)]
pub struct FramebufferQuad<'a, const N: usize, X: AttachmentWithoutExtra<Tex = FlatTexture>> {
    vertex_array: &'static VertexArray<QuadVertex>,
    texture: &'a Framebuffer<N, X>,
}

impl<'a, const N: usize, X: AttachmentWithoutExtra<Tex = FlatTexture>> FramebufferQuad<'a, N, X> {
    pub fn new_from_framebuffer(framebuffer: &'a Framebuffer<N, X>) -> Self {
        let vertex_array = screen_quad();
        Self {
            vertex_array,
            texture: framebuffer,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError<'a> {
    #[allow(clippy::allow_attributes, reason = "rustc only allows allow")]
    #[allow(unused, reason = "Error")]
    VertexArrayDraw(VertexArrayDrawError<'a>),
}
error_boilerplate!(<'a> DrawError<'a>);

impl<'a, const N: usize, X: AttachmentWithoutExtra<Tex = FlatTexture>> FramebufferQuad<'a, N, X> {
    pub(crate) fn draw<const OUT: usize, O, I>(
        self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'a, Quad<N>, O, I::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, I>,
    ) -> Result<(), DrawError<'a>>
    where
        I: FramebufferWithoutExtra<OUT>,
    {
        _ = active_shader.cull_face(CullFace::BackFace);

        let input = self
            .texture
            .get_textures_raw()
            .iter()
            .map(|tex|
                // SAFETY: this `tex` is only borrowed here, is never passed to another function and the reference expires at the end of the function
                unsafe {
                tex.try_borrow_unguarded()
                    .expect("cannot write to a framebuffer and read at the same time, if this panics, so does the gpu")
            })
            .enumerate()
            .map(|(index, tex)| {
                (
                    CString::new(format!("in_texture{index}")).expect("should be no Nul"),
                    tex as &dyn Texture,
                )
            });

        active_shader.register_texture(input);

        self.vertex_array
            .draw(active_shader, active_framebuffer)
            .map_err(DrawError::VertexArrayDraw)?;

        Ok(())
    }
}
