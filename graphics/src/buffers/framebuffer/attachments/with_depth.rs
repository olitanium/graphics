use core::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use super::WithoutExtra;
use crate::buffers::framebuffer::traits::{AttachmentTextureInfo, OptTexBuilderMap};
use crate::buffers::framebuffer::{
    self,
    Attachment,
    AttachmentWithDepth,
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

/// Marker type to hold a framebuffer's depth buffer
pub struct WithDepth {
    texture: Rc<RefCell<FlatTexture>>,
    builder_map: OptTexBuilderMap<Self>,
}

impl fmt::Debug for WithDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(WithDepth))
            .field("texture", &self.texture)
            .finish_non_exhaustive()
    }
}

impl Attachment for WithDepth {
    type Tex = FlatTexture;
    type TexBuilder = FlatTextureBuilder<FramebufferAttachment<Self>>;

    fn new(size: (TexDim, TexDim), builder_map: OptTexBuilderMap<Self>) -> Self {
        let mut texture_builder = FlatTexture::builder().depth_attachment(size);

        if let Some(map) = &builder_map {
            texture_builder = map(texture_builder);
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
            internal_format: gl::DEPTH_COMPONENT,
            format: gl::DEPTH_COMPONENT,
            data_type: gl::UNSIGNED_INT,
            attachment: gl::DEPTH_ATTACHMENT,
        })
    }

    fn new_framebuffer<const N: usize>(&mut self, size: (TexDim, TexDim)) -> Framebuffer<N, Self> {
        let mut builder = framebuffer::Builder::new_flat().depth().size(size);
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
        context.stencil_testing(false);
        context.srgb_framebuffer(false);
    }
}

impl AttachmentWithoutExtra for WithDepth {}
impl AttachmentWithDepth for WithDepth {
    fn get_texture(&self) -> Rc<RefCell<FlatTexture>> {
        self.texture.clone()
    }

    unsafe fn get_texture_ref(&self) -> &FlatTexture {
        unsafe { self.texture.try_borrow_unguarded() }.unwrap()
    }
}

// TODO: Check whether these Into are needed
impl Into<WithoutExtra> for WithDepth {
    fn into(self) -> WithoutExtra {
        unimplemented!()
    }
}
