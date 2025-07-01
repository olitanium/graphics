pub(crate) mod element_array_buffer;
mod error;
pub use error::Error;

pub(crate) mod framebuffer;

pub use framebuffer::traits as fb_traits;

pub use framebuffer::{
    flat_builder as framebuffer_builder,
    AttachmentTextureInfo,
    ActiveFramebuffer,
    Builder,
    DefaultFramebuffer,
    Framebuffer,
    WithDepth,
    CubeWithDepth,
    WithStencil,
    CubeWithoutExtra,
    WithoutExtra,
    FramebufferWithDepth,
};

pub use vertex_array::VertexArray;
pub use vertex_array::IncompleteVertex;

pub(crate) mod vertex_array;
pub(crate) mod vertex_buffer;