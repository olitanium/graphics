use core::cell::RefMut;

/// Helpful traits to ensure that certain draw operations cannot be applied to
/// framebuffer's which are missing particular attachments
use super::{ActiveFramebuffer, Framebuffer, FramebufferContext};
use crate::texture::{Texture, TextureHasBuilder};
use crate::types::{self, FrameBufferId, TexDim};

#[derive(Debug)]
pub struct AttachmentTextureInfo {
    pub internal_format: types::GLenum,
    pub format: types::GLenum,
    pub data_type: types::GLenum,
    pub attachment: types::GLenum,
}

pub type OptTexBuilderMap<B: AttachmentWithoutExtra> =
    Option<Box<dyn Fn(B::TexBuilder) -> B::TexBuilder + 'static>>;

pub trait AttachmentWithoutExtra: Sized {
    type Tex: Texture + TextureHasBuilder;
    type TexBuilder;

    fn new(size: (TexDim, TexDim), builder_map: OptTexBuilderMap<Self>) -> Self;

    fn components() -> Option<AttachmentTextureInfo>;

    /// Shortcut function to make a framebuffer based on an attachment
    /// Used to assist in resizing a framebuffer.
    fn new_framebuffer<const N: usize>(&mut self, size: (TexDim, TexDim)) -> Framebuffer<N, Self>;

    /// Get size of the internal buffer, as a fallback if there is no
    /// colourSized buffer. Ideally this would move into WithDepthTrait
    /// along with `aspect_ratio`.
    fn size(&self) -> (TexDim, TexDim);

    /// Enable the various OpenGL drawing modes.
    fn enables(context: &mut FramebufferContext);
}

/// Marker trait for attachments which provide a depth buffer
pub trait AttachmentWithDepth: AttachmentWithoutExtra {
    // fn get_texture(&self) -> Rc<RefCell<Self::Tex>>;

    /// # Safety
    /// This method's reference must not live long enough that the texture can
    /// be mutably borrowed by the underlying RefCell
    fn get_texture_ref(&self) -> &Self::Tex;

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

pub trait FramebufferInternals: Sized {
    type Tex: Texture;

    fn size(&self) -> (TexDim, TexDim);
    fn aspect_ratio(&self) -> f32 {
        let (x, y) = self.size();
        #[allow(clippy::cast_precision_loss)]
        (x.as_primitive() as f32 / y.as_primitive() as f32)
    }
    fn id(&self) -> &FrameBufferId;

    fn enables(&self, context: &mut FramebufferContext);
}

impl<T> FramebufferWithoutExtra<1> for T
where
    T: FramebufferWithoutExtra<2>,
{
    fn try_give_texture_lock(&self) -> Option<[RefMut<'_, Self::Tex>; 1]> {
        let [a, ..] = self.try_give_texture_lock()?;
        Some([a])
    }
}
impl<T> FramebufferWithDepth<1> for T
where
    T: FramebufferWithDepth<2>,
{
    fn depth_testing(depth_testing: bool, context: &mut FramebufferContext) {
        T::depth_testing(depth_testing, context);
    }
}
impl<T> FramebufferWithStencil<1> for T
where
    T: FramebufferWithStencil<2>,
{
    fn stencil_testing(stencil_testing: bool, context: &mut FramebufferContext) {
        T::stencil_testing(stencil_testing, context);
    }
}

impl<T> FramebufferWithoutExtra<2> for T
where
    T: FramebufferWithoutExtra<3>,
{
    fn try_give_texture_lock(&self) -> Option<[RefMut<'_, Self::Tex>; 2]> {
        let [a, b, ..] = self.try_give_texture_lock()?;
        Some([a, b])
    }
}
impl<T> FramebufferWithDepth<2> for T
where
    T: FramebufferWithDepth<3>,
{
    fn depth_testing(depth_testing: bool, context: &mut FramebufferContext) {
        T::depth_testing(depth_testing, context);
    }
}
impl<T> FramebufferWithStencil<2> for T
where
    T: FramebufferWithStencil<3>,
{
    fn stencil_testing(stencil_testing: bool, context: &mut FramebufferContext) {
        T::stencil_testing(stencil_testing, context);
    }
}
pub trait FramebufferWithoutExtra<const OUT: usize>: FramebufferInternals {
    fn bind<'a, 'b>(
        &'a self,
        fb_context: &'b mut FramebufferContext,
    ) -> ActiveFramebuffer<'a, 'b, OUT, Self> {
        fb_context.register(self)
    }

    fn try_give_texture_lock(&self) -> Option<[RefMut<'_, Self::Tex>; OUT]>;
}

pub trait FramebufferWithDepth<const OUT: usize>: FramebufferWithoutExtra<OUT> {
    fn depth_testing(depth_testing: bool, context: &mut FramebufferContext);
}
pub trait FramebufferWithStencil<const OUT: usize>: FramebufferWithDepth<OUT> {
    fn stencil_testing(stencil_testing: bool, context: &mut FramebufferContext);
}
