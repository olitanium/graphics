use std::cell::RefCell;
use std::rc::Rc;

use crate::framebuffer::traits::{
    Attachment,
    AttachmentWithDepth,
    AttachmentWithStencil,
    AttachmentWithoutExtra,
    FramebufferInternals,
    FramebufferWithDepth,
    FramebufferWithStencil,
    FramebufferWithoutExtra,
};
use crate::gl_call;
use crate::texture::{FlatTexture, Texture};
use crate::types::{FrameBufferId, TexDim, ToPrimitive};

mod active_framebuffer;
mod builder;
mod size;
pub mod traits;

pub use active_framebuffer::{ActiveFramebuffer, FramebufferContext};
pub use builder::Builder;
use builder::MissingSize;

pub fn flat_builder<const N: usize>() -> Builder<N, MissingSize, WithoutExtra> {
    Builder::new_flat()
}
pub fn cubic_builder<const N: usize>() -> Builder<N, MissingSize, CubeWithoutExtra> {
    Builder::new_cubic()
}

pub mod attachments;
use attachments::{CubeWithDepth, CubeWithoutExtra, WithStencil, WithoutExtra};

// A `Framebuffer` is a destination for drawing a scene, the default
// Framebuffer is accessible after an `Environment` is initialzed and is for
// drawing to the screen. Any other frameuffers will draw to a colour buffer
// (or HDR buffer) which can be retreived with `Framebuffer::get_colour`

/// The screen framebuffer, its size is that of the window. It cannot store
/// colour values outside the range [0.0, 1.0]
#[derive(Debug)]
pub struct DefaultFramebuffer {
    size: (TexDim, TexDim),
}

impl Drop for DefaultFramebuffer {
    fn drop(&mut self) {}
}

impl FramebufferInternals<1> for DefaultFramebuffer {
    type Tex = FlatTexture;

    fn id(&self) -> <FrameBufferId as ToPrimitive>::Primitive {
        0
    }

    fn size(&self) -> (TexDim, TexDim) {
        self.size
    }

    fn enables(context: &mut FramebufferContext) {
        context.depth_testing(true);
        context.stencil_testing(true);
        context.srgb_framebuffer(true);
    }
}

impl FramebufferWithoutExtra<1> for DefaultFramebuffer {}
impl FramebufferWithDepth<1> for DefaultFramebuffer {
    fn depth_testing(depth_testing: bool, context: &mut FramebufferContext) {
        // `DefaultFramebuffer` is effectively a `WithStencil` buffer
        WithStencil::depth_testing(depth_testing, context);
    }
}
impl FramebufferWithStencil<1> for DefaultFramebuffer {
    fn stencil_testing(stencil_testing: bool, context: &mut FramebufferContext) {
        // `DefaultFramebuffer` is effectively a `WithStencil` buffer
        WithStencil::stencil_testing(stencil_testing, context);
    }
}

/// A Framebuffer for drawing anything onto. Can be any size (rectangle). As an
/// HDR buffer it can store any value in it's four channels (rgba)
#[derive(Debug)]
pub struct Framebuffer<const OUT: usize, X: Attachment> {
    /// OpenGL ID for this Framebuffer
    id: FrameBufferId,
    /// Holder for Stencil and/or Depth buffers.
    /// To ensure no Stencil without depth (which would be an incomplete buffer)
    stencil_or_depth: X,
    /// A default screen-size quad whose texture(s) are those drawn onto.
    textures: [Rc<RefCell<X::Tex>>; OUT],
    // quad: Quad<OUT>,
}

impl<const OUT: usize, X: Attachment> Drop for Framebuffer<OUT, X> {
    fn drop(&mut self) {
        gl_call! {
            gl::DeleteFramebuffers(1, &self.id());
        }
    }
}

impl<const OUT: usize, X: Attachment> FramebufferInternals<OUT> for Framebuffer<OUT, X> {
    type Tex = X::Tex;

    fn id(&self) -> <FrameBufferId as ToPrimitive>::Primitive {
        self.id.to_primitive()
    }

    fn size(&self) -> (TexDim, TexDim) {
        self.textures
            .get(0)
            .map_or_else(|| self.stencil_or_depth.size(), |tex| tex.borrow().size())
    }

    fn enables(context: &mut FramebufferContext) {
        X::enables(context)
    }
}

impl<const OUT: usize, X: AttachmentWithoutExtra> FramebufferWithoutExtra<OUT>
    for Framebuffer<OUT, X>
{
}
impl<const OUT: usize, X: AttachmentWithDepth> FramebufferWithDepth<OUT> for Framebuffer<OUT, X> {
    fn depth_testing(depth_testing: bool, context: &mut FramebufferContext) {
        X::depth_testing(depth_testing, context);
    }
}
impl<const OUT: usize, X: AttachmentWithStencil> FramebufferWithStencil<OUT>
    for Framebuffer<OUT, X>
{
    fn stencil_testing(stencil_testing: bool, context: &mut FramebufferContext) {
        X::stencil_testing(stencil_testing, context);
    }
}

impl DefaultFramebuffer {
    /// Internal function to generate the default (screen) `FrameBuffer`
    // pub(crate) fn new(window: &Window) -> Self {
    pub(crate) fn new(size: (TexDim, TexDim)) -> Self {
        // let size = window.get_framebuffer_size();
        // let size = (TexDim::new(size.0), TexDim::new(size.1));

        Self { size }
    }
}

impl Default for DefaultFramebuffer {
    fn default() -> Self {
        Self {
            size: (TexDim::new(1), TexDim::new(1)),
        }
    }
}

impl<const N: usize, X: Attachment> Framebuffer<N, X> {
    /// Access one of the Framebuffer draw locations,
    ///
    /// # Errors
    /// Returns Error if `index` >= `N` i.e. out of bounds
    pub fn get_colour(&self, index: usize) -> Option<Rc<RefCell<X::Tex>>> {
        self.textures.get(index).cloned()
    }

    /// Return all textures, in order, of the Framebuffer.
    pub fn get_all_colour(&self) -> [Rc<RefCell<X::Tex>>; N] {
        self.textures.clone()
    }
}

// impl<const N: usize, X: Attachment<Tex = FlatTexture>> Framebuffer<N, X> {
// pub fn quad(&self) -> Quad<N> {
// Quad::screen(self.get_all_colour())
// }
// }

impl<const N: usize, D: AttachmentWithDepth> Framebuffer<N, D> {
    pub fn get_attachment_texture(&self) -> Rc<RefCell<D::Tex>> {
        self.stencil_or_depth.get_texture()
    }

    pub unsafe fn get_attachment_ref(&self) -> &dyn Texture {
        unsafe { self.stencil_or_depth.get_texture_ref() }
    }
}
