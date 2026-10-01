use core::cell::RefCell;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::{array, fmt};

use super::attachments::{CubeWithoutExtra, WithDepth, WithStencil, WithoutExtra};
use super::traits::OptTexBuilderMap;
use super::{CubeWithDepth, Framebuffer};
use crate::framebuffer::traits::AttachmentWithoutExtra;
use crate::gl_call;
use crate::texture::{
    TexBuilder as _,
    TexBuilderCanBuild as _,
    Texture as _,
    TextureHasBuilder as _,
};
use crate::types::{self, FrameBufferId, TexDim};

#[derive(Debug, Default)]
pub struct MissingSize;

#[derive(Debug)]
pub struct HasSize((TexDim, TexDim));

/// Framebuffer builder type, used to select the number of colour buffers and
/// types of depth testing.
#[derive(Default)]
pub struct Builder<const N: usize, S, B: AttachmentWithoutExtra> {
    /// Size of the underlying texture.
    size: S,
    /// What type of internal.
    _phantom_buffer: PhantomData<B>,
    map_attachment: OptTexBuilderMap<B>,
}

impl<const N: usize, S: Debug, B: AttachmentWithoutExtra + Debug> Debug for Builder<N, S, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("size", &self.size)
            .field("_phantom_buffer", &self._phantom_buffer)
            .field(
                "map_attachment",
                if self.map_attachment.is_some() {
                    &"Some(..)"
                } else {
                    &"None"
                },
            )
            .finish_non_exhaustive()
    }
}

impl<const N: usize> Builder<N, MissingSize, WithoutExtra> {
    pub fn new_flat() -> Self {
        Self {
            size: MissingSize,
            _phantom_buffer: PhantomData,
            map_attachment: None,
        }
    }
}

impl<const N: usize> Builder<N, MissingSize, CubeWithoutExtra> {
    pub fn new_cubic() -> Self {
        Self {
            size: MissingSize,
            _phantom_buffer: PhantomData,
            map_attachment: None,
        }
    }
}

impl<const N: usize, S> Builder<N, S, CubeWithoutExtra> {
    pub fn cubic_depth(self) -> Builder<N, S, CubeWithDepth> {
        Builder {
            _phantom_buffer: PhantomData,
            map_attachment: None,
            ..self
        }
    }
}

impl<const N: usize, B: AttachmentWithoutExtra> Builder<N, MissingSize, B> {
    /// Set the framebuffer size.
    pub fn size(self, size: (TexDim, TexDim)) -> Builder<N, HasSize, B> {
        Builder {
            size: HasSize(size),
            ..self
        }
    }
}

impl<const N: usize, S> Builder<N, S, WithoutExtra> {
    /// enable depth testing.
    pub fn depth(self) -> Builder<N, S, WithDepth> {
        Builder {
            _phantom_buffer: PhantomData,
            map_attachment: None,
            ..self
        }
    }
}

impl<const N: usize, S> Builder<N, S, WithDepth> {
    /// enable stencil testing. This method is only available after enabling
    /// depth testing with `Builder::depth`.
    pub fn stencil(self) -> Builder<N, S, WithStencil> {
        Builder {
            _phantom_buffer: PhantomData,
            map_attachment: None,
            ..self
        }
    }
}

impl<const N: usize, B: AttachmentWithoutExtra> Builder<N, HasSize, B> {
    #[must_use]
    pub fn map_attachment<F: Fn(B::TexBuilder) -> B::TexBuilder + 'static>(
        self,
        map_attachment: F,
    ) -> Self {
        Self {
            map_attachment: Some(Box::new(map_attachment)),
            ..self
        }
    }

    /// Build from information given to the `Builder` before.
    #[must_use]
    #[inline]
    pub fn build(self) -> Framebuffer<N, B> {
        let id = {
            let mut id = 0;
            gl_call! { gl::GenFramebuffers(1, &raw mut id); }
            FrameBufferId::new(id)
        };

        gl_call! {
            gl::BindFramebuffer(gl::FRAMEBUFFER, id.as_primitive());
        }

        let colour: [_; N] = array::from_fn(|_| {
            RefCell::new(
                B::Tex::builder()
                    .size(self.size.0)
                    // TODO: Add `Map` field to implement this map
                    //.wrap_s_t((gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE))
                    .build(),
            )
        });

        #[expect(
            clippy::cast_possible_truncation,
            reason = "There will not be more than GLenum::MAX colour attachments"
        )]
        for (index, texture) in colour.iter().enumerate() {
            gl_call! {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + index as types::GLenum,
                    gl::TEXTURE_2D,
                    texture.borrow().id().as_primitive(),
                    0,
                );
            }
        }

        if N == 0 {
            gl_call! { gl::DrawBuffer(gl::NONE); }
        } else {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "There will not be more than GLenum::MAX colour attachments"
            )]
            let draw_buffers: [_; N] =
                array::from_fn(|x| gl::COLOR_ATTACHMENT0 + x as types::GLenum);

            gl_call! {
                gl::DrawBuffers(
                    #[expect( clippy::cast_possible_truncation, reason = "There will not be more than GLenum::MAX colour attachments")]
                    #[expect( clippy::cast_possible_wrap, reason = "There will not be more than GLenum::MAX colour attachments")]
                    (N as types::GLsizei),
                    draw_buffers.as_ptr()
                );
            }
        }

        let stencil_or_depth = B::new(self.size.0, self.map_attachment);

        Framebuffer {
            id,
            stencil_or_depth,
            textures: colour,
        }
    }
}
