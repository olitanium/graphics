use std::sync::LazyLock;

use builder::MissingData;
use colour::ColourRGB;

use super::{Texture, TextureHasBuilder};
use crate::gl_call;
use crate::types::{TexDim, TexId, };

mod builder;
pub use builder::{Builder as CubeMapBuilder, FramebufferAttachment as CubeFrameBufferAttachment};

#[derive(Debug)]
pub struct CubeMap {
    id: TexId,
    size: TexDim,
}

impl CubeMap {
    pub fn zeroes() -> Self {
        Self::grayscale(0.0)
    }

    pub fn white() -> Self {
        Self::grayscale(1.0)
    }

    pub fn grayscale(shade: f32) -> Self {
        Self::monochrome(ColourRGB::new([shade, shade, shade]))
    }

    pub fn monochrome(colour: ColourRGB) -> Self {
        Self::builder().monochrome(colour).build()
    }
}

impl Default for CubeMap {
    fn default() -> Self {
        Self::zeroes()
    }
}

impl TextureHasBuilder for CubeMap {
    type Builder = CubeMapBuilder<MissingData>;

    fn builder() -> Self::Builder {
        Self::Builder::new()
    }
}

impl Texture for CubeMap {
    /// # Safety
    /// The id returned must not be allowed to drop the graphics card texture
    fn id(&self) -> &TexId {
        &self.id
    }

    fn dyn_blank() -> &'static dyn Texture {
        static DEFAULT_CUBEMAP: LazyLock<CubeMap> = LazyLock::new(CubeMap::default);

        &*DEFAULT_CUBEMAP
    }

    fn bind_to(&self, index: u32) {
        gl_call! {
            gl::ActiveTexture(gl::TEXTURE0 + index);
        }
        gl_call! {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        gl_call! {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id.to_primitive());
        }
    }

    fn size(&self) -> (TexDim, TexDim) {
        (self.size, self.size)
    }
}
