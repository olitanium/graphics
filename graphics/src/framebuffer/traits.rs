use std::cell::RefCell;
use std::rc::Rc;

/// Helpful traits to ensure that certain draw operations cannot be applied to
/// framebuffer's which are missing particular attachments
use super::{ActiveFramebuffer, Framebuffer, FramebufferContext};
use crate::texture::{Texture, TextureHasBuilder};
use crate::types::{self, FrameBufferId, TexDim, };

#[derive(Debug)]
pub struct AttachmentTextureInfo {
    pub internal_format: types::GLenum,
    pub format: types::GLenum,
    pub data_type: types::GLenum,
    pub attachment: types::GLenum,
}

pub type OptTexBuilderMap<B: Attachment> =
    Option<Box<dyn Fn(B::TexBuilder) -> B::TexBuilder + 'static>>;

pub trait Attachment: Sized {
    type Tex: Texture + TextureHasBuilder;
    type TexBuilder;

    fn new(size: (TexDim, TexDim), builder_map: OptTexBuilderMap<Self>) -> Self;

    fn components() -> Option<AttachmentTextureInfo>;

    /// Shortcut function to make a framebuffer based on an attachment
    /// Used to assist in resizing a framebuffer.
    // TODO: this method is technically destructive on self. Can the signature
    // be updated to reflect this?
    fn new_framebuffer<const N: usize>(&mut self, size: (TexDim, TexDim)) -> Framebuffer<N, Self>;

    /// Get size of the internal buffer, as a fallback if there is no
    /// colourSized buffer. Ideally this would move into WithDepthTrait
    /// along with `aspect_ratio`.
    fn size(&self) -> (TexDim, TexDim);

    /// Enable the various OpenGL drawing modes.
    fn enables(context: &mut FramebufferContext);
}

/// Marker trait for attachments which do not provide any new buffers
pub trait AttachmentWithoutExtra: Attachment {}

/// Marker trait for attachments which provide a depth buffer
pub trait AttachmentWithDepth: AttachmentWithoutExtra {
    fn get_texture(&self) -> Rc<RefCell<Self::Tex>>;
    /// # Safety
    /// This method's reference must not live long enough that the texture can be mutably borrowed by the
    /// underlying RefCell
    unsafe fn get_texture_ref(&self) -> &Self::Tex;

    fn depth_testing(depth_testing: bool, context: &mut FramebufferContext) {
        context.depth_testing(depth_testing)
    }
}

/// Marker trait for attachments which provide a depth-stencil buffer
pub trait AttachmentWithStencil: AttachmentWithDepth {
    fn stencil_testing(stencil_testing: bool, context: &mut FramebufferContext) {
        context.stencil_testing(stencil_testing)
    }
}

pub trait FramebufferInternals<const OUT: usize>: Sized {
    type Tex: Texture;

    fn size(&self) -> (TexDim, TexDim);
    fn aspect_ratio(&self) -> f32 {
        let (x, y) = self.size();
        x.to_primitive() as f32 / y.to_primitive() as f32
    }
    fn id(&self) -> &FrameBufferId;

    fn enables(context: &mut FramebufferContext);

    fn bind<'a, 'b>(
        &'a self,
        register: &'b mut FramebufferContext,
    ) -> ActiveFramebuffer<'a, 'b, OUT, Self> {
        register.register(self)
    }
}

pub trait FramebufferWithoutExtra<const OUT: usize>: FramebufferInternals<OUT> {}

pub trait FramebufferWithDepth<const OUT: usize>: FramebufferWithoutExtra<OUT> {
    fn depth_testing(depth_testing: bool, context: &mut FramebufferContext);
}

pub trait FramebufferWithStencil<const OUT: usize>: FramebufferWithDepth<OUT> {
    fn stencil_testing(stencil_testing: bool, context: &mut FramebufferContext);
}
