use std::convert::Infallible;
use std::ops::{ControlFlow, FromResidual, Residual, Try};

use crate::framebuffer::FramebufferContext;
use crate::shader_program::ShaderProgramContext;
use crate::types::Query;

pub enum PollResult<'a, E> {
    Draw(Box<dyn Draw + 'a>),
    Error(E),
    Close,
    Restart,
}

pub enum PollResultResidual<E> {
    Close,
    Err(E),
    Restart,
}

impl<'a, E> Residual<Box<dyn Draw + 'a>> for PollResultResidual<E> {
    type TryType = PollResult<'a, E>;
}

impl<E> FromResidual<Result<Infallible, E>> for PollResult<'_, E> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(e) => Self::Error(e),
        }
    }
}

impl<E> FromResidual<Result<!, E>> for PollResult<'_, E> {
    fn from_residual(residual: Result<!, E>) -> Self {
        let e = residual.into_err();
        Self::Error(e)
    }
}

impl<'a, E> Try for PollResult<'a, E> {
    type Output = Box<dyn Draw + 'a>;
    type Residual = PollResultResidual<E>;

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Error(err) => ControlFlow::Break(PollResultResidual::Err(err)),
            Self::Close => ControlFlow::Break(PollResultResidual::Close),
            Self::Draw(draw) => ControlFlow::Continue(draw),
            Self::Restart => ControlFlow::Break(PollResultResidual::Restart),
        }
    }

    fn from_output(output: Self::Output) -> Self {
        Self::Draw(output)
    }
}

impl<E> FromResidual<PollResultResidual<E>> for PollResult<'_, E> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            PollResultResidual::Err(err) => Self::Error(err),
            PollResultResidual::Close => Self::Close,
            PollResultResidual::Restart => Self::Restart,
        }
    }
}

pub trait Draw {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut;
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
    pub const OK: Self = Self::Draw;
}

impl<T: Draw + ?Sized> Draw for [Box<T>] {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut outputs = Vec::new();
        for group in self {
            match group.draw(fb_context, shader_marker) {
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
            DrawOut::MultiQueryOutput {
                _token: PrivateToken,
                values: outputs,
            }
        }
    }
}
impl<'a> Draw for Vec<Box<dyn Draw + 'a>> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let slice = self.into_boxed_slice();
        slice.draw(fb_context, sp_context)
    }
}

#[derive(Debug)]
pub struct DrawNothing;

impl DrawNothing {
    pub fn new() -> Box<Self> {
        Box::new(DrawNothing)
    }
}

impl Draw for DrawNothing {
    fn draw(self: Box<Self>, _: &mut FramebufferContext, _: &mut ShaderProgramContext) -> DrawOut {
        DrawOut::OK
    }
}
