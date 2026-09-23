
use crate::framebuffer::FramebufferContext;
use crate::types::Query;
use crate::{Draw, DrawOut, ShaderProgramContext, gl_call};

#[derive(Debug)]
pub struct HasDrawnQuery<D: Draw + ?Sized> {
    name: usize,
    query: Query,
    target: Box<D>,
}

impl<D: Draw + ?Sized> HasDrawnQuery<D> {
    #[must_use]
    pub fn new(name: usize, target: Box<D>) -> Box<Self> {
        let query = {
            let mut value = 0;
            gl_call! {gl::GenQueries(1, &raw mut value) };
            Query::new(value)
        };
        Box::new(Self {
            name,
            query,
            target,
        })
    }
}

impl<D: Draw + ?Sized> Draw for HasDrawnQuery<D> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }

    #[inline]
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut
    where
        Self: Sized,
    {
        gl_call! {gl::BeginQuery(gl::ANY_SAMPLES_PASSED, self.query.as_primitive()); }

        _ = self.target.draw(fb_context, sp_context);

        gl_call! {gl::EndQuery(gl::ANY_SAMPLES_PASSED);}

        DrawOut::make_query(self.name, self.query)
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        let primitive = self.as_primitive();
        gl_call! {gl::DeleteQueries(1, &raw const primitive);}
    }
}

impl Query {
    #[must_use]
    pub fn answer(self) -> bool {
        self.answer_ref()
    }

    pub fn answer_ref(&self) -> bool {
        let mut did_succeed = 0;
        gl_call! {gl::GetQueryObjectiv(self.as_primitive(), gl::QUERY_RESULT, &raw mut did_succeed)}

        did_succeed == gl::TRUE.into()
    }
}
