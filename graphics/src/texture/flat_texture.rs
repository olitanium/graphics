use core::fmt::Debug;
use core::num::NonZero;
use std::sync::LazyLock;

use colour::ColourRGBA;

use super::{Texture, TextureHasBuilder};
use crate::gl_call;
use crate::types::{TexDim, TexId};

mod builder;
use builder::MissingData;
pub use builder::{Builder as FlatTextureBuilder, FramebufferAttachment};

#[derive(Debug)]
pub struct FlatTexture {
    id: TexId,
    size: (TexDim, TexDim),
}

impl FlatTexture {
    pub fn zeroes() -> Self {
        Self::grayscale(0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::grayscale(1.0, 1.0)
    }

    pub fn grayscale(shade: f32, alpha: f32) -> Self {
        Self::monochrome(ColourRGBA::new([shade, shade, shade, alpha]))
    }

    pub fn monochrome(colour: ColourRGBA) -> Self {
        Self::builder().monochrome(colour).build()
    }
}

impl Drop for FlatTexture {
    fn drop(&mut self) {
        let primitive = self.id.as_primitive();
        gl_call! { gl::DeleteTextures(1, &raw const primitive); }
    }
}

impl Default for FlatTexture {
    fn default() -> Self {
        const {
            Self {
                id: TexId::new(0),
                size: (
                    TexDim::from_non_zero(NonZero::new(1).expect("one")),
                    TexDim::from_non_zero(NonZero::new(1).expect("one")),
                ),
            }
        }
    }
}

impl TextureHasBuilder for FlatTexture {
    type Builder = FlatTextureBuilder<MissingData>;

    fn builder() -> <Self as TextureHasBuilder>::Builder {
        <Self as TextureHasBuilder>::Builder::new()
    }
}

impl Texture for FlatTexture {
    fn dyn_blank() -> &'static dyn Texture {
        static DEFAULT_FLAT_TEXTURE: LazyLock<FlatTexture> = LazyLock::new(FlatTexture::default);

        &*DEFAULT_FLAT_TEXTURE
    }

    fn id(&self) -> &TexId {
        &self.id
    }

    fn bind_to(&self, index: u32) {
        gl_call! {
            gl::ActiveTexture(gl::TEXTURE0 + index);
        }
        gl_call! {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
        }
        gl_call! {
            gl::BindTexture(gl::TEXTURE_2D, self.id.as_primitive());
        }
    }

    fn size(&self) -> (TexDim, TexDim) {
        self.size
    }
}
