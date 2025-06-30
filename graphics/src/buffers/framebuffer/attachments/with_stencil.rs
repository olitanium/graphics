use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::buffers::framebuffer::traits::{AttachmentTextureInfo, OptTexBuilderMap};
use crate::buffers::framebuffer::{
    self,
    Attachment,
    AttachmentWithDepth,
    AttachmentWithStencil,
    AttachmentWithoutExtra,
    Framebuffer,
    FramebufferContext,
};
use crate::texture::{
    FlatTexture,
    FlatTextureBuilder,
    FramebufferAttachment,
    Texture,
    TextureHasBuilder,
};
use crate::types::TexDim;

/// Marker type to hold a framebuffer's depth and stencil textures.
pub struct WithStencil {
    texture: Rc<RefCell<FlatTexture>>,
    builder_map: OptTexBuilderMap<Self>,
}

impl fmt::Debug for WithStencil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(WithStencil))
            .field("texture", &self.texture)
            .finish_non_exhaustive()
    }
}

impl Attachment for WithStencil {
    type Tex = FlatTexture;
    type TexBuilder = FlatTextureBuilder<FramebufferAttachment<Self>>;

    fn new(size: (TexDim, TexDim), builder_map: OptTexBuilderMap<Self>) -> Self {
        let mut texture_builder = FlatTexture::builder().stencil_attachment(size);

        if let Some(builder_map) = &builder_map {
            texture_builder = builder_map(texture_builder);
        }

        let texture = texture_builder
            .build()
            .expect("This is not a WithoutExtra texture");

        Self {
            texture: Rc::new(RefCell::new(texture)),
            builder_map,
        }
    }

    fn components() -> Option<AttachmentTextureInfo> {
        Some(AttachmentTextureInfo {
            internal_format: gl::DEPTH24_STENCIL8,
            format: gl::DEPTH_STENCIL,
            data_type: gl::UNSIGNED_INT_24_8,
            attachment: gl::DEPTH_STENCIL_ATTACHMENT,
        })
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
        self.texture.borrow().size()
    }

    fn enables(context: &mut FramebufferContext) {
        context.depth_testing(true);
        context.stencil_testing(true);
        context.srgb_framebuffer(false);
    }
}

impl AttachmentWithoutExtra for WithStencil {}
impl AttachmentWithDepth for WithStencil {
    fn get_texture(&self) -> Rc<RefCell<FlatTexture>> {
        self.texture.clone()
    }

    unsafe fn get_texture_ref(&self) -> &FlatTexture {
        unsafe { self.texture.try_borrow_unguarded() }.unwrap()
    }
}
impl AttachmentWithStencil for WithStencil {}
