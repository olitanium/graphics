use std::{iter, mem};

use super::Framebuffer;
use super::traits::Attachment;
use crate::types::TexDim;

impl<const N: usize, D: Attachment> Framebuffer<N, D> {
    /// Resize the colour and optional attachements to new size
    // TODO: enable destructive use of `self.stencil_or_depth`
    pub fn resize(&mut self, size: (TexDim, TexDim)) {
        // Create a new framebuffer with the correct dims, then swap that into the
        // location of the original
        let mut new = self.stencil_or_depth.new_framebuffer(size);

        // This new contains newRc<newCell>, I want this to become oldRc<newCell> before
        // it is swapped into self
        // mem::swap(&mut new.data.textures, &mut self.data.textures);

        for (old_tex, new_tex) in iter::zip(&new.textures, &self.textures) {
            old_tex.swap(new_tex);
        }

        mem::swap(self, &mut new);
    }
}
