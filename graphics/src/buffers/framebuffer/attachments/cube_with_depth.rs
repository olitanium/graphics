use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use super::WithDepth;
use crate::buffers::framebuffer::traits::{
    AttachmentWithDepth,
    AttachmentWithoutExtra,
    OptTexBuilderMap,
};
use crate::buffers::framebuffer::{self, Attachment, FramebufferContext};
use crate::buffers::{AttachmentTextureInfo, Framebuffer};
use crate::texture::{
    CubeFrameBufferAttachment,
    CubeMap,
    CubeMapBuilder,
    Texture,
    TextureHasBuilder,
};
use crate::types::TexDim;

pub struct CubeWithDepth {
    texture: Rc<RefCell<CubeMap>>,
    builder_map: OptTexBuilderMap<Self>,
}

impl Debug for CubeWithDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(CubeWithDepth))
            .field("texture", &self.texture)
            .finish_non_exhaustive()
    }
}

impl Attachment for CubeWithDepth {
    type Tex = CubeMap;
    type TexBuilder = CubeMapBuilder<CubeFrameBufferAttachment<Self>>;

    fn new(size: (TexDim, TexDim), builder_map: OptTexBuilderMap<Self>) -> Self {
        let mut builder = CubeMap::builder().depth_attachment(size.0);

        if let Some(builder_map) = &builder_map {
            builder = builder_map(builder);
        }

        let cube_map = builder.build().expect("Contains Depth");

        Self {
            texture: Rc::new(RefCell::new(cube_map)),
            builder_map,
        }
    }

    fn components() -> Option<AttachmentTextureInfo> {
        Some(AttachmentTextureInfo {
            internal_format: gl::DEPTH_COMPONENT,
            format: gl::DEPTH_COMPONENT,
            data_type: gl::FLOAT,
            attachment: gl::DEPTH_ATTACHMENT,
        })
    }

    fn enables(context: &mut FramebufferContext) {
        WithDepth::enables(context)
    }

    fn size(&self) -> (TexDim, TexDim) {
        self.texture.borrow().size()
    }

    fn new_framebuffer<const N: usize>(&mut self, size: (TexDim, TexDim)) -> Framebuffer<N, Self> {
        let mut builder = framebuffer::Builder::new_cubic().cubic_depth().size(size);
        if let Some(builder_map) = self.builder_map.take() {
            builder = builder.map_attachment(builder_map);
        }
        builder.build()
    }
}

impl AttachmentWithoutExtra for CubeWithDepth {}

impl AttachmentWithDepth for CubeWithDepth {
    fn get_texture(&self) -> Rc<RefCell<Self::Tex>> {
        self.texture.clone()
    }

    unsafe fn get_texture_ref(&self) -> &Self::Tex {
        unsafe { self.texture.try_borrow_unguarded() }.unwrap()
    }
}
