use std::{iter, mem};

use super::traits::Attachment;
use super::{DefaultFramebuffer, Framebuffer};
use crate::texture::Texture;
use crate::types::{TexDim, ToPrimitive};

impl<const N: usize, D: Attachment> Framebuffer<N, D> {
    /// Get the size of the framebuffer
    /// # Panics
    /// Panics when attachement is `WithoutExtra` and there are zero colour
    /// buffers i.e. `N == 0`
    pub fn size(&self) -> (TexDim, TexDim) {
        self.get_colour(0)
            .map_or_else(|_| self.stencil_or_depth.size(), |tex| tex.borrow().size())
    }

    /// Calculate the aspect ratio (x/y)
    pub fn aspect_ratio(&self) -> f32 {
        let (x, y) = self.size();
        x.to_primitive() as f32 / y.to_primitive() as f32
    }

    /// Resize the colour and optional attachements to new size
    // TODO: enable destructive use of `self.data.stencil_or_depth`
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

impl DefaultFramebuffer {
    /// Get the size of the framebuffer
    pub fn size(&self) -> (TexDim, TexDim) {
        self.size
    }

    /// Calculate the aspect ratio (x/y)
    pub fn aspect_ratio(&self) -> f32 {
        let (x, y) = self.size();
        x.to_primitive() as f32 / y.to_primitive() as f32
    }
}
