use crate::buffers::framebuffer::traits::OptTexBuilderMap;
use crate::buffers::framebuffer::{
    self,
    Attachment,
    AttachmentWithoutExtra,
    Framebuffer,
    FramebufferContext,
};
use crate::buffers::AttachmentTextureInfo;
use crate::texture::FlatTexture;
use crate::types::TexDim;

/// Marker type to hold no extra depth or stencil buffer
#[derive(Debug, Default)]
pub struct WithoutExtra;

impl Attachment for WithoutExtra {
    type Tex = FlatTexture;
    type TexBuilder = ();

    fn new(_: (TexDim, TexDim), _: OptTexBuilderMap<Self>) -> Self {
        Self
    }

    fn components() -> Option<AttachmentTextureInfo> {
        None
    }

    fn new_framebuffer<const OUT: usize>(
        &mut self,
        size: (TexDim, TexDim),
    ) -> Framebuffer<OUT, Self> {
        framebuffer::Builder::new_flat().size(size).build()
    }

    fn size(&self) -> (TexDim, TexDim) {
        unimplemented!(
            "Reaching this panic means that the finding size of a zero colour, zero attachment \
             Framebuffer has been attempted"
        )
    }

    fn enables(context: &mut FramebufferContext) {
        context.depth_testing(false);
        context.stencil_testing(false);
        context.srgb_framebuffer(false);
    }
}

impl AttachmentWithoutExtra for WithoutExtra {}
