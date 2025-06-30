pub(crate) mod element_array_buffer;
mod error;
pub use error::Error;

pub(crate) mod framebuffer;

pub use framebuffer::{
    flat_builder as framebuffer_builder,
    AttachmentTextureInfo,
    Builder,
    DefaultFramebuffer,
    Framebuffer,
    WithDepth,
    WithStencil,
    WithoutExtra,
};

pub(crate) mod vertex_array;
pub(crate) mod vertex_buffer;
