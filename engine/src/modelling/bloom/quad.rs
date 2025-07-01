use std::cell::RefCell;
use std::rc::Rc;

use graphics::Framebuffer;
use graphics::buffers::WithoutExtra;
use crate::modelling::quad::Quad;
use graphics::texture::{FlatTexture, Texture};
use graphics::types::TexDim;
pub use graphics::buffers::Builder as FrameBufferBuilder;
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

        // let blur_x = ShaderProgram::builder()
        // .vertex_shader_raw(include_bytes!("../../../shaders/blur_x/blur_x.vert"))
        // .expect(EXPECT_MESSAGE)
        // .fragment_shader_raw(include_bytes!("../../../shaders/blur_x/blur_x.frag"))
        // .expect(EXPECT_MESSAGE)
        // .build();

        let framebuffer_x = FrameBufferBuilder::new_flat().size(size).build();

        // let blur_y_merge = ShaderProgram::builder()
        // .vertex_shader_raw(include_bytes!(
        // "../../../shaders/blur_y_merge/blur_y_merge.vert"
        // ))
        // .expect(EXPECT_MESSAGE)
        // .fragment_shader_raw(include_bytes!(
        // "../../../shaders/blur_y_merge/blur_y_merge.frag"
        // ))
        // .expect(EXPECT_MESSAGE)
        // .build();

        Self {
            to_blur,
            // blur_x,
            framebuffer_x,
            // blur_y_merge,
        }
    }

    pub fn resize(&mut self, size: (TexDim, TexDim)) {
        self.framebuffer_x.resize(size)
    }
}
