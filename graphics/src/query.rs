use std::error::Error;
use std::marker::PhantomData;

use crate::framebuffer::attachments::WithDepth;
use crate::framebuffer::{Framebuffer, FramebufferContext};
use crate::types::Query;
use crate::{ApiResult, Draw, DrawOut, ShaderProgramContext, gl_call};

pub struct HasDrawnQuery<E: std::error::Error, D: Draw<E>> {
    name: usize,
    query: Query,
    target: Box<D>,
    _error: PhantomData<fn() -> E>,
}

impl<E: std::error::Error, D: Draw<E>> HasDrawnQuery<E, D> {
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
            _error: PhantomData,
        })
    }
}

impl<E: std::error::Error, D: Draw<E>> Draw<E> for HasDrawnQuery<E, D> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, E> {
        gl_call! {gl::BeginQuery(gl::ANY_SAMPLES_PASSED, self.query.as_primitive()); }

        let _ = self.target.draw(fb_context, sp_context)?;

        gl_call! {gl::EndQuery(gl::ANY_SAMPLES_PASSED);}

        let out = DrawOut::QueryOutput {
            _token: crate::PrivateToken,
            query_object: self.query,
            name: self.name,
        };
        Ok(out)
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        let primitive = self.as_primitive();
        gl_call! {gl::DeleteQueries(1, &raw const primitive);}
    }
}

impl Query {
    pub fn get_answer(&self) -> bool {
        let mut did_succeed = 0;
        gl_call! {gl::GetQueryObjectiv(self.as_primitive(), gl::QUERY_RESULT, &raw mut did_succeed)}

        did_succeed == gl::TRUE.into()
    }
}
