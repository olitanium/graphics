use std::mem;

use super::Framebuffer;
use crate::framebuffer::traits::AttachmentWithoutExtra;
use crate::types::TexDim;

impl<const N: usize, D: AttachmentWithoutExtra> Framebuffer<N, D> {
    // Resize the colour and optional attachements to new size
    // TODO: enable destructive use of `self.stencil_or_depth`
    pub fn resize(&mut self, size: (TexDim, TexDim)) {
        // Create a new framebuffer with the correct dims, then swap that into the
        // location of the original

        let mut new = self.stencil_or_depth.new_framebuffer(size);
        mem::swap(&mut new, self);
    }
}
