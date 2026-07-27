pub use graphics::framebuffer::Builder;
use graphics::framebuffer::Framebuffer;
use graphics::framebuffer::attachments::WithoutExtra;
use graphics::types::TexDim;

#[derive(Debug)]
/// A pseudo-quad which takes the output of a framebuffer and blurs.
/// Works in conjuntion with the `BloomGroup` to take output from a
/// Framebuffer<2> to produce a blurred texture
pub struct Bloom {
    /// framebuffer in which to draw the x-blurred image
    pub(crate) framebuffer_x: Framebuffer<2, WithoutExtra>,
}

impl Bloom {
    pub fn new(size: (TexDim, TexDim)) -> Self {
        let framebuffer_x = Builder::new_flat().size(size).build();

        Self {
            // to_blur,
            framebuffer_x,
        }
    }

    // pub fn new_from_framebuffer<X: AttachmentWithoutExtra<Tex = FlatTexture>>(
    // buffer: &Framebuffer<3, X>,
    // ) -> Self {
    // let [all, light, _dark] = buffer.get_all_colour();
    // Self::new(all, light)
    // }

    // TODO: resize
    pub fn resize(&mut self, size: (TexDim, TexDim)) {
        self.framebuffer_x.resize(size)
    }
}
