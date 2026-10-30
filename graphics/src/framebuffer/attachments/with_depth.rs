use core::fmt;

use colour::ColourRGBA;

use crate::framebuffer::traits::{AttachmentTextureInfo, OptTexBuilderMap};
use crate::framebuffer::{
    self,
    AttachmentWithDepth,
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
    WrapType,
};
use crate::types::TexDim;

/// Type to hold a framebuffer's depth buffer.
pub struct WithDepth {
    texture: FlatTexture,
    builder_map: OptTexBuilderMap<Self>,
}

impl fmt::Debug for WithDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(WithDepth))
            .field("texture", &self.texture)
            .finish_non_exhaustive()
    }
}

impl FramebufferAttachment for WithDepth {
    type Tex = FlatTexture;
    type TexBuilder = FlatTextureBuilder<FlatFramebufferAttachment<Self>>;

    fn new(size: (TexDim, TexDim), builder_map: OptTexBuilderMap<Self>) -> Self {
        let mut texture_builder = FlatTexture::builder()
            .depth_attachment(size)
            .wrap_s_t(WrapType::ClampToEdge);

        if let Some(map) = &builder_map {
            texture_builder = map(texture_builder);
        }

        let texture = texture_builder.build();

        Self {
            texture,
            builder_map,
        }
    }

    fn new_framebuffer<const N: usize>(&mut self, size: (TexDim, TexDim)) -> Framebuffer<N, Self> {
        let mut builder = framebuffer::Builder::new_flat().depth().size(size);
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
        context.stencil_testing(false);
        context.srgb_framebuffer(false);
    }
}

impl AttachmentWithDepth for WithDepth {
    // fn get_texture(&self) -> Rc<RefCell<FlatTexture>> {
    // self.texture.clone()
    // }
    //

    fn components() -> AttachmentTextureInfo {
        AttachmentTextureInfo {
            internal_format: gl::DEPTH_COMPONENT,
            format: gl::DEPTH_COMPONENT,
            data_type: gl::UNSIGNED_INT,
            attachment: gl::DEPTH_ATTACHMENT,
        }
    }

    fn get_texture_ref(&self) -> &FlatTexture {
        &self.texture
    }
}
