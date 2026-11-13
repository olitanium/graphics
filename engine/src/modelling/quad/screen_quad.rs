mod group;

use alloc::ffi::CString;

use graphics::framebuffer::traits::{
    AttachmentWithDepth,
    FramebufferAttachment,
    FramebufferWithoutExtra,
};
use graphics::framebuffer::{ActiveFramebuffer, Framebuffer};
use graphics::shader_program::{CullFace, ShaderSetUniformError};
use graphics::texture::{FlatTexture, Texture};
use graphics::{ActiveShaderProgram, vertex_array};
pub use group::Group as ScreenQuadGroup;
use utils::error_boilerplate;

use crate::modelling::{Quad, test_models};

#[derive(Debug)]
pub struct FramebufferQuad<'a, const N: usize, X: FramebufferAttachment<Tex = FlatTexture>> {
    texture: &'a Framebuffer<N, X>,
}

impl<'a, const N: usize, X: FramebufferAttachment<Tex = FlatTexture>> FramebufferQuad<'a, N, X> {
    pub const fn new_from_framebuffer(framebuffer: &'a Framebuffer<N, X>) -> Self {
        Self {
            texture: framebuffer,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError {
    #[allow(clippy::allow_attributes, reason = "rustc only allows allow")]
    #[allow(unused, reason = "Error")]
    VertexArrayDraw(vertex_array::DrawError),
    #[allow(clippy::allow_attributes, reason = "rustc only allows allow")]
    #[allow(unused, reason = "Error")]
    TextureBindError(ShaderSetUniformError),
}
error_boilerplate!(DrawError);

impl<'a, const N: usize, X: FramebufferAttachment<Tex = FlatTexture>> FramebufferQuad<'a, N, X> {
    pub(crate) fn draw<const OUT: usize, O, I>(
        &self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'a, Quad<N>, O, I::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, I>,
    ) -> Result<(), DrawError>
    where
        I: FramebufferWithoutExtra<OUT>,
    {
        _ = active_shader.cull_face(CullFace::BackFace);

        let input = self
            .texture
            .get_textures_raw()
            .iter()
            .enumerate()
            // cannot write to a framebuffer and read at the same time, in order to prevent gpu
            // crash, this filters out any illegal access.
            .filter_map(|(index, tex)| {
                // SAFETY: this `tex` is only borrowed here, and the reference
                // expires at the end of the function
                let res_ref = unsafe { tex.try_borrow_unguarded() };
                if let Err(x) = &res_ref {
                    eprintln!(
                        "texture which would have been `in_texture{index}` cannot be bound \
                         because the texture `RefCell` is already borrowed: {x:?}"
                    );
                }
                res_ref.ok().map(|reftex| (index, reftex))
            })
            // `usize::display` will not return a NUL character, so this always passes
            .filter_map(|(index, tex)| {
                CString::new(format!("in_texture{index}"))
                    .map(|cstr| (cstr, tex as &dyn Texture))
                    .ok()
            });

        active_shader
            .register_texture(input)
            .map_err(DrawError::TextureBindError)?;

        test_models::screen_quad()
            .draw(active_shader, active_framebuffer)
            .map_err(DrawError::VertexArrayDraw)?;

        Ok(())
    }
}

impl<const N: usize, X> FramebufferQuad<'_, N, X>
where
    X: AttachmentWithDepth<Tex = FlatTexture>,
{
    pub(crate) fn get_depth(&self) -> &dyn Texture {
        self.texture.get_attachment_ref()
    }
}
