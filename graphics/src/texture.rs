use crate::types::{TexDim, TexId};

mod cubemap;
// mod error;
// pub use error::Error;
mod flat_texture;

pub use cubemap::CubeMap;
pub(crate) use cubemap::{CubeFramebufferAttachment, CubeMapBuilder};
pub use flat_texture::FlatTexture;
pub(crate) use flat_texture::{FlatFramebufferAttachment, FlatTextureBuilder};
mod parameters;
pub use parameters::{Magnification, Minification, MipMapInfo, WrapType};
mod image;
pub use image::{Image, NewImageError};

const fn get_format<const N: usize>() -> (u32, u32) {
    match N {
        0 => panic!(),
        1 => (gl::R16F, gl::RED),
        2 => (gl::RG16F, gl::RG),
        3 => (gl::RGB16F, gl::RGB),
        4 => (gl::RGBA16F, gl::RGBA),
        5.. => panic!(),
    }
}

pub trait Texture {
    fn dyn_blank() -> &'static dyn Texture
    where
        Self: Sized;

    fn bind_to(&self, index: u32);

    fn size(&self) -> (TexDim, TexDim);

    fn id(&self) -> &TexId;
}

pub trait TextureHasBuilder: Texture {
    type Builder: TexBuilder<ExpectedFinal = Self>;

    fn builder() -> Self::Builder;
}

pub trait TexBuilder {
    type ExpectedFinal: Texture;
    type ReadyToBuild<const N: usize>: TexBuilderCanBuild<N, Output = Self::ExpectedFinal>;

    fn size<const N: usize>(self, size: (TexDim, TexDim)) -> Self::ReadyToBuild<N>;
}

pub trait TexBuilderCanBuild<const N: usize> {
    type Output: Texture;

    fn build(self) -> Self::Output;
}
