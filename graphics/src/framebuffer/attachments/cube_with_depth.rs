use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt::{self, Debug};

use super::WithDepth;
use crate::framebuffer::traits::{
    AttachmentTextureInfo,
    AttachmentWithDepth,
    AttachmentWithoutExtra,
    OptTexBuilderMap,
};
use crate::framebuffer::{self, Framebuffer, FramebufferContext};
use crate::texture::{
    CubeFrameBufferAttachment,
    CubeMap,
    CubeMapBuilder,
    Texture as _,
    TextureHasBuilder as _,
};
use crate::types::TexDim;

pub struct CubeWithDepth {
    texture: Rc<RefCell<CubeMap>>,
    builder_map: OptTexBuilderMap<Self>,
}

impl Debug for CubeWithDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(CubeWithDepth))
            .field("texture", &self.texture)
            .finish_non_exhaustive()
    }
}

impl AttachmentWithoutExtra for CubeWithDepth {
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
        WithDepth::enables(context);
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

impl AttachmentWithDepth for CubeWithDepth {
    // fn get_texture(&self) -> Rc<RefCell<<Self as
    // AttachmentWithoutExtra>::Tex>> { self.texture.clone()
    // }
    //
    fn get_texture_ref(&self) -> &<Self as AttachmentWithoutExtra>::Tex {
        unsafe { self.texture.try_borrow_unguarded() }.unwrap()
    }
}
