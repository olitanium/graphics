use core::cell::{BorrowMutError, RefMut};
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashSet;

use utils::error_boilerplate;

use super::{FramebufferWithDepth, FramebufferWithStencil};
use crate::framebuffer::traits::FramebufferWithoutExtra;
use crate::types::{FrameBufferId, TexDim};
use crate::{gl_call, types};

#[derive(Debug)]
pub struct FramebufferContext {
    current_framebuffer: FrameBufferId::Primitive,
    viewport: (TexDim, TexDim),
    cleared: HashSet<types::GLuint>,
}
static IS_INIT: AtomicBool = AtomicBool::new(false);

macro_rules! enable_disable {
    ($name:ident, $glenum:ident) => {
        pub fn $name(&mut self, $name: bool) {
            if $name {
                gl_call! { gl::Enable(gl::$glenum); }
            } else {
                gl_call! { gl::Disable(gl::$glenum); }
            }
        }
    };
}

impl FramebufferContext {
    enable_disable!(depth_testing, DEPTH_TEST);

    enable_disable!(stencil_testing, STENCIL_TEST);

    enable_disable!(srgb_framebuffer, FRAMEBUFFER_SRGB);

    pub fn new() -> Option<Self> {
        let is_init = IS_INIT.swap(false, Ordering::Relaxed);
        (!is_init).then(|| Self {
            current_framebuffer: 0,
            viewport: const { (TexDim::new(1), TexDim::new(1)) },
            cleared: HashSet::new(),
        })
    }

    pub fn register<'a, 'b, const OUT: usize, D>(
        &'b mut self,
        framebuffer: &'a D,
    ) -> Result<ActiveFramebuffer<'a, 'b, OUT, D>, BindError>
    where
        D: FramebufferWithoutExtra<OUT>,
    {
        if self.cleared.insert(framebuffer.id().as_primitive()) {
            let mut active = ActiveFramebuffer::new(framebuffer, self)?;
            active.clear();
            Ok(active)
        } else {
            ActiveFramebuffer::new(framebuffer, self)
        }
    }

    pub fn clear(&mut self) {
        self.cleared.clear();
    }

    /// Clear the currently bound Framebuffer and ALL buffer bits.
    pub fn clear_current_framebuffer(&mut self) {
        gl_call! {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }
}

#[derive(Debug)]
pub struct ActiveFramebuffer<'a, 'b, const OUT: usize, D: FramebufferWithoutExtra<OUT>> {
    framebuffer: PhantomData<&'a D>,
    _texture_locks: [RefMut<'a, D::Tex>; OUT],
    context: &'b mut FramebufferContext,
}

#[derive(Debug)]
pub enum BindError {
    ReadingFromFramebuffer(BorrowMutError),
}
error_boilerplate!(BindError);

impl<'a, 'b, const OUT: usize, D: FramebufferWithoutExtra<OUT>> ActiveFramebuffer<'a, 'b, OUT, D> {
    pub fn new(framebuffer: &'a D, context: &'b mut FramebufferContext) -> Result<Self, BindError> {
        let (width, height) = framebuffer.size();

        let fb_primitive = framebuffer.id().as_primitive();
        if fb_primitive != context.current_framebuffer {
            context.current_framebuffer = fb_primitive;
            gl_call! {
                gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.id().as_primitive());
            }
        }
        if (width, height) != context.viewport {
            context.viewport = (width, height);
            gl_call! {
                gl::Viewport(0, 0, width.as_primitive(), height.as_primitive());
            }
        }

        framebuffer.enables(context);

        let texture_locks = framebuffer
            .try_give_texture_lock()
            .map_err(BindError::ReadingFromFramebuffer)?;

        Ok(Self {
            framebuffer: PhantomData,
            _texture_locks: texture_locks,
            context,
        })
    }

    /// Clear the currently bound Framebuffer and ALL buffer bits.
    pub fn clear(&mut self) {
        self.context.clear_current_framebuffer();
    }
}

impl<const OUT: usize, X: FramebufferWithDepth<OUT>> ActiveFramebuffer<'_, '_, OUT, X> {
    pub fn depth_testing(&mut self, depth_testing: bool) {
        X::depth_testing(depth_testing, self.context);
    }
}

impl<const OUT: usize, X: FramebufferWithStencil<OUT>> ActiveFramebuffer<'_, '_, OUT, X> {
    pub fn stencil_testing(&mut self, stencil_testing: bool) {
        X::stencil_testing(stencil_testing, self.context);
    }
}
