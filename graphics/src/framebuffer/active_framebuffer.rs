use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Mutex;

use super::{FramebufferInternals, FramebufferWithDepth, FramebufferWithStencil};
use crate::gl_call;
use crate::types::{FrameBufferId};

#[derive(Debug)]
pub struct FramebufferContext {
    cleared: HashSet<gl::types::GLuint>,
}
static IS_INIT: Mutex<bool> = Mutex::new(false);

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
        let mut is_init = IS_INIT.lock().ok()?;
        if *is_init {
            None
        } else {
            *is_init = true;
            Some(Self {
                cleared: HashSet::new(),
            })
        }
    }

    pub fn register<'a, 'b, const OUT: usize, D: FramebufferInternals<OUT>>(
        &'b mut self,
        framebuffer: &'a D,
    ) -> ActiveFramebuffer<'a, 'b, OUT, D> {
        if self.cleared.insert(framebuffer.id().to_primitive()) {
            let mut active = ActiveFramebuffer::new(framebuffer, self);
            active.clear();
            active
        } else {
            ActiveFramebuffer::new(framebuffer, self)
        }
    }

    pub fn clear(&mut self) {
        self.cleared.clear();
    }

    /// Clear the currently bound Framebuffer and ALL buffer bits
    pub fn clear_framebuffer(&mut self) {
        gl_call! {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }
}

#[derive(Debug)]
pub struct ActiveFramebuffer<'a, 'b, const OUT: usize, D: FramebufferInternals<OUT>> {
    _framebuffer: PhantomData<&'a D>,
    context: &'b mut FramebufferContext,
}

impl<'a, 'b, const OUT: usize, D: FramebufferInternals<OUT>> ActiveFramebuffer<'a, 'b, OUT, D> {
    pub fn new(framebuffer: &'a D, context: &'b mut FramebufferContext) -> Self {
        let (width, height) = framebuffer.size();

        gl_call! {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.id().to_primitive());
        }
        gl_call! {
            gl::Viewport(0, 0, width.to_primitive(), height.to_primitive());
        }

        D::enables(context);

        Self {
            _framebuffer: PhantomData,
            context,
        }
    }

    /// Clear the currently bound Framebuffer and ALL buffer bits
    pub fn clear(&mut self) {
        self.context.clear_framebuffer()
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
