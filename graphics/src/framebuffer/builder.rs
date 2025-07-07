use std::array;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use super::attachments::{CubeWithoutExtra, WithDepth, WithStencil, WithoutExtra};
use super::traits::OptTexBuilderMap;
use super::{Attachment, CubeWithDepth, Framebuffer};
use crate::gl_call;
use crate::texture::{TexBuilder, TexBuilderCanBuild, Texture, TextureHasBuilder};
use crate::types::{self, FrameBufferId, TexDim};

#[derive(Debug, Default)]
pub struct MissingSize;

#[derive(Debug)]
pub struct HasSize((TexDim, TexDim));

/// Framebuffer builder type, used to select the number of colour buffers and
/// types of depth testing
#[derive(Default)]
pub struct Builder<const N: usize, S, B: Attachment> {
    /// Size of the underlying texture
    size: S,
    /// What type of internal
    _phantom_buffer: PhantomData<B>,
    map_attachment: OptTexBuilderMap<B>,
}

impl<const N: usize, S: Debug, B: Attachment + Debug> Debug for Builder<N, S, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder")
            .field("size", &self.size)
            .field("_phantom_buffer", &self._phantom_buffer)
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

impl<const N: usize, B: Attachment> Builder<N, MissingSize, B> {
    /// Set the framebuffer size
    pub fn size(self, size: (TexDim, TexDim)) -> Builder<N, HasSize, B> {
        Builder {
            size: HasSize(size),
            ..self
        }
    }
}

impl<const N: usize, S> Builder<N, S, WithoutExtra> {
    /// enable depth testing
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
    /// depth testing with Builder::depth
    pub fn stencil(self) -> Builder<N, S, WithStencil> {
        Builder {
            _phantom_buffer: PhantomData,
            map_attachment: None,
            ..self
        }
    }
}

impl<const N: usize, B: Attachment> Builder<N, HasSize, B> {
    pub fn map_attachment<F: Fn(B::TexBuilder) -> B::TexBuilder + 'static>(
        self,
        map_attachment: F,
    ) -> Self {
        Self {
            map_attachment: Some(Box::new(map_attachment)),
            ..self
        }
    }
}

impl<const N: usize, B: Attachment> Builder<N, HasSize, B> {
    /// Build from information given to the `Builder` before.
    pub fn build(self) -> Framebuffer<N, B> {
        let id = {
            let mut id = 0;
            gl_call! { gl::GenFramebuffers(1, &mut id); }
            FrameBufferId::new(id)
        };

        gl_call! {
            gl::BindFramebuffer(gl::FRAMEBUFFER, id.to_primitive());
        }

        let colour: [Rc<RefCell<B::Tex>>; N] = array::from_fn(|_| {
            Rc::new(RefCell::new(
                B::Tex::builder()
                    .size(self.size.0)
                    // TODO: Add `Map` field to implement this map
                    //.wrap_s_t((gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE))
                    .build(),
            ))
        });

        for (index, texture) in colour.iter().enumerate() {
            gl_call! {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + index as types::GLenum,
                    gl::TEXTURE_2D,
                    texture.borrow().id().to_primitive(),
                    0,
                );
            }
        }

        if N == 0 {
            gl_call! { gl::DrawBuffer(gl::NONE); }
        } else {
            let draw_buffers: [types::GLenum; N] =
                array::from_fn(|x| gl::COLOR_ATTACHMENT0 + x as types::GLenum);

            gl_call! { gl::DrawBuffers(N as types::GLsizei, draw_buffers.as_ptr()); }
        }

        let stencil_or_depth = B::new(self.size.0, self.map_attachment);

        Framebuffer {
            id,
            stencil_or_depth,
            textures: colour,
        }
    }
}
