use core::fmt;

use crate::framebuffer::traits::{AttachmentTextureInfo, OptTexBuilderMap};
use crate::framebuffer::{
    self,
    AttachmentWithDepth,
    AttachmentWithStencil,
    Framebuffer,
    FramebufferAttachment,
    FramebufferContext,
};
use crate::texture::{
    FlatFramebufferAttachment,
    FlatTexture,
    FlatTextureBuilder,
    Texture as _,
    TextureHasBuilder as _,
};
use crate::types::TexDim;

/// Marker type to hold a framebuffer's depth and stencil textures.
pub struct WithStencil {
    texture: FlatTexture,
    builder_map: OptTexBuilderMap<Self>,
}

impl fmt::Debug for WithStencil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(WithStencil))
            .field("texture", &self.texture)
            .finish_non_exhaustive()
    }
}

impl FramebufferAttachment for WithStencil {
    type Tex = FlatTexture;
    type TexBuilder = FlatTextureBuilder<FlatFramebufferAttachment<Self>>;

    fn new(size: (TexDim, TexDim), builder_map: OptTexBuilderMap<Self>) -> Self {
        let mut texture_builder = FlatTexture::builder().stencil_attachment(size);

        if let Some(builder_map) = &builder_map {
            texture_builder = builder_map(texture_builder);
        }

        let texture = texture_builder.build();

        Self {
            texture,
            builder_map,
        }
    }

    fn new_framebuffer<const OUT: usize>(
        &mut self,
        size: (TexDim, TexDim),
    ) -> Framebuffer<OUT, Self> {
        let mut builder = framebuffer::Builder::new_flat()
            .depth()
            .stencil()
            .size(size);
        // TODO: once signature takes `self` as receiver, remove `.take()`
        if let Some(builder_map) = self.builder_map.take() {
            builder = builder.map_attachment(builder_map);
        }
        builder.build()
    }

    fn size(&self) -> (TexDim, TexDim) {
        self.texture.size()
    }

    fn enables(context: &mut FramebufferContext) {
        context.depth_testing(true);
        context.stencil_testing(true);
        context.srgb_framebuffer(false);
    }
}

impl AttachmentWithDepth for WithStencil {
    fn components() -> AttachmentTextureInfo {
        AttachmentTextureInfo {
            internal_format: gl::DEPTH24_STENCIL8,
            format: gl::DEPTH_STENCIL,
            data_type: gl::UNSIGNED_INT_24_8,
            attachment: gl::DEPTH_STENCIL_ATTACHMENT,
        }
    }

    fn get_texture_ref(&self) -> &FlatTexture {
        &self.texture
    }
}
impl AttachmentWithStencil for WithStencil {}
