use super::WithoutExtra;
use crate::framebuffer::traits::{AttachmentTextureInfo, OptTexBuilderMap};
use crate::framebuffer::{
    self,
    Attachment,
    AttachmentWithoutExtra,
    Framebuffer,
    FramebufferContext,
};
use crate::texture::{CubeFrameBufferAttachment, CubeMap, CubeMapBuilder};
use crate::types::TexDim;

#[derive(Debug)]
pub struct CubeWithoutExtra;

impl Attachment for CubeWithoutExtra {
    type Tex = CubeMap;
    type TexBuilder = CubeMapBuilder<CubeFrameBufferAttachment<Self>>;

    fn new(_: (TexDim, TexDim), _: OptTexBuilderMap<Self>) -> Self {
        Self
    }

    fn components() -> Option<AttachmentTextureInfo> {
        None
    }

    fn enables(context: &mut FramebufferContext) {
        WithoutExtra::enables(context)
    }

    fn size(&self) -> (TexDim, TexDim) {
        unimplemented!(
            "Reaching this panic means that the size of a zero colour, zero attachment \
             Framebuffer has been attempted. This value is hence undefined"
        )
    }

    fn new_framebuffer<const N: usize>(&mut self, size: (TexDim, TexDim)) -> Framebuffer<N, Self> {
        framebuffer::Builder::new_cubic().size(size).build()
    }
}

impl AttachmentWithoutExtra for CubeWithoutExtra {}
