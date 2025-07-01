use crate::types::{TexDim, TexId};

mod cubemap;
mod error;
pub use error::Error;
mod flat_texture;
//mod material;

pub use cubemap::CubeMap;
pub(crate) use cubemap::{CubeFrameBufferAttachment, CubeMapBuilder};
pub use flat_texture::FlatTexture;
pub(crate) use flat_texture::{FlatTextureBuilder, FramebufferAttachment};
//pub use material::Material;
mod parameters;
pub use parameters::{Magnification, Minification, MipMapInfo, WrapType};

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
    type ReadyToBuild: TexBuilderCanBuild<Output = Self::ExpectedFinal>;

    fn size(self, size: (TexDim, TexDim)) -> Self::ReadyToBuild;
}

pub trait TexBuilderCanBuild {
    type Output: Texture;

    fn build(self) -> Self::Output;
}
