use crate::ApiResult;
use crate::framebuffer::FramebufferContext;
use crate::query::HasDrawnQuery;
use crate::shader_program::ShaderProgramContext;
use crate::types::Query;

pub trait Draw<E> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, E>;
}

pub(crate) struct PrivateToken;

#[must_use]
pub enum DrawOut {
    Draw,
    QueryOutput {
        name: usize,
        #[expect(private_interfaces)]
        _token: PrivateToken,
        query_object: Query,
    },
    MultiQueryOutput {
        #[expect(private_interfaces)]
        _token: PrivateToken,
        values: Vec<(usize, Query)>,
    },
}

impl DrawOut {
    pub const OK<NEVER>: ApiResult<Self, NEVER> = Ok(Self::Draw);
}

impl<T: Draw<E> + ?Sized, E> Draw<E> for [Box<T>] {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, E> {
        let mut outputs = Vec::new();
        for group in self {
            match group.draw(fb_context, shader_marker)? {
                DrawOut::QueryOutput {
                    name, query_object, ..
                } => {
                    outputs.push((name, query_object));
                }
                DrawOut::MultiQueryOutput { values, .. } => {
                    outputs.extend(values);
                }
                _ => {}
            }
        }

        if outputs.is_empty() {
            DrawOut::OK
        } else {
            Ok(DrawOut::MultiQueryOutput {
                _token: PrivateToken,
                values: outputs,
            })
        }
    }
}
// impl<E: Error> Draw<E> for Vec<Box<dyn Draw<E>>> {
// fn draw(
// self: Box<Self>,
// fb_context: &mut FramebufferContext,
// shader_marker: &mut ShaderProgramContext,
// ) {
// for group in *self {
// group.draw(fb_context, shader_marker);
// }
// }
// }

pub struct DrawNothing;

impl<E: std::error::Error> Draw<E> for DrawNothing {
    fn draw(
        self: Box<Self>,
        _: &mut FramebufferContext,
        _: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, E> {
        Ok(DrawOut::Draw)
    }
}
