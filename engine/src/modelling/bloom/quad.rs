use std::cell::RefCell;
use std::rc::Rc;

pub use graphics::framebuffer::Builder;
use graphics::framebuffer::Framebuffer;
use graphics::framebuffer::attachments::WithoutExtra;
use graphics::framebuffer::traits::Attachment;
use graphics::texture::{FlatTexture, Texture};
use graphics::types::TexDim;

use crate::modelling::quad::Quad;
#[derive(Debug)]
/// A pseudo-quad which takes the output of a framebuffer and blurs.
/// Works in conjuntion with the `BloomGroup` to take output from a
/// Framebuffer<2> to produce a blurred texture
pub struct Bloom {
    /// derives it's textures from an HDR framebuffer
    pub(crate) to_blur: Quad<2>,
    /// framebuffer in which to draw the x-blurred image
    pub(crate) framebuffer_x: Framebuffer<2, WithoutExtra>,
}

impl Bloom {
    pub fn new(dark: Rc<RefCell<FlatTexture>>, light: Rc<RefCell<FlatTexture>>) -> Self {
        let size = dark.borrow().size();
        let to_blur = Quad::screen([dark, light]);

        let framebuffer_x = Builder::new_flat().size(size).build();

        Self {
            to_blur,
            framebuffer_x,
        }
    }

    pub fn new_from_framebuffer<X: Attachment<Tex = FlatTexture>>(
        buffer: &Framebuffer<3, X>,
    ) -> Self {
        let [all, light, _dark] = buffer.get_all_colour();
        Self::new(all, light)
    }

    pub fn resize(&mut self, size: (TexDim, TexDim)) {
        self.framebuffer_x.resize(size)
    }
}
